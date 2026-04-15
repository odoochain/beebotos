#!/usr/bin/env node
/**
 * WhatsApp Baileys Bridge for BeeBotOS
 * 
 * This Node.js bridge connects BeeBotOS to WhatsApp using the Baileys library.
 * It communicates with Rust via stdio (JSON-RPC style messages).
 * 
 * Features:
 * - QR code authentication
 * - Multi-device support
 * - Auto-reconnect
 * - Message deduplication
 * - Media download/upload
 * - Group support
 */

const { 
    default: makeWASocket, 
    DisconnectReason, 
    useMultiFileAuthState,
    fetchLatestBaileysVersion,
    makeCacheableSignalKeyStore,
    downloadMediaMessage,
    proto
} = require('@whiskeysockets/baileys');
const qrcode = require('qrcode-terminal');
const pino = require('pino');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

// Configuration
const CONFIG = {
    authDir: process.env.WHATSAPP_AUTH_DIR || './auth_info_baileys',
    reconnectInterval: parseInt(process.env.WHATSAPP_RECONNECT_INTERVAL) || 5000,
    maxReconnectAttempts: parseInt(process.env.WHATSAPP_MAX_RECONNECT_ATTEMPTS) || 10,
    mediaDownloadDir: process.env.WHATSAPP_MEDIA_DIR || './media/downloads',
    logLevel: process.env.WHATSAPP_LOG_LEVEL || 'info',
    printQR: process.env.WHATSAPP_PRINT_QR !== 'false',
};

// Logger setup
const logger = pino({
    level: CONFIG.logLevel,
    transport: {
        target: 'pino-pretty',
        options: {
            colorize: true,
            translateTime: true
        }
    }
});

// Global state
let sock = null;
let reconnectAttempts = 0;
let isConnected = false;
let connectionState = 'disconnected';
let messageQueue = [];
let processedMessages = new Set(); // For deduplication
const MAX_PROCESSED_MESSAGES = 1000;

// Ensure directories exist
if (!fs.existsSync(CONFIG.authDir)) {
    fs.mkdirSync(CONFIG.authDir, { recursive: true });
}
if (!fs.existsSync(CONFIG.mediaDownloadDir)) {
    fs.mkdirSync(CONFIG.mediaDownloadDir, { recursive: true });
}

/**
 * Send message to Rust via stdout
 */
function sendToRust(message) {
    const json = JSON.stringify(message);
    process.stdout.write(json + '\n');
}

/**
 * Log message to stderr (so it doesn't interfere with JSON communication)
 */
function log(level, message, meta = {}) {
    logger[level](meta, message);
}

/**
 * Generate unique message ID for deduplication
 */
function generateMessageId(msg) {
    const data = `${msg.key.remoteJid}:${msg.key.id}:${msg.messageTimestamp}`;
    return crypto.createHash('sha256').update(data).digest('hex').substring(0, 32);
}

/**
 * Check if message is a duplicate
 */
function isDuplicate(msg) {
    const id = generateMessageId(msg);
    if (processedMessages.has(id)) {
        return true;
    }
    processedMessages.add(id);
    
    // Keep set size manageable
    if (processedMessages.size > MAX_PROCESSED_MESSAGES) {
        const iterator = processedMessages.values();
        processedMessages.delete(iterator.next().value);
    }
    return false;
}

/**
 * Download media from message
 */
async function downloadMedia(msg) {
    try {
        const buffer = await downloadMediaMessage(
            msg,
            'buffer',
            {},
            {
                logger,
                reuploadRequest: sock.updateMediaMessage
            }
        );
        
        if (!buffer) {
            return null;
        }

        // Generate filename
        const timestamp = Date.now();
        const extension = getMediaExtension(msg);
        const filename = `${timestamp}_${msg.key.id}.${extension}`;
        const filepath = path.join(CONFIG.mediaDownloadDir, filename);
        
        // Save file
        fs.writeFileSync(filepath, buffer);
        
        return {
            path: filepath,
            filename: filename,
            size: buffer.length,
            mimetype: getMediaMimetype(msg)
        };
    } catch (error) {
        log('error', 'Failed to download media', { error: error.message });
        return null;
    }
}

/**
 * Get media extension from message
 */
function getMediaExtension(msg) {
    const message = msg.message;
    if (message?.imageMessage) return 'jpg';
    if (message?.videoMessage) return 'mp4';
    if (message?.audioMessage) return message.audioMessage.ptt ? 'ogg' : 'mp3';
    if (message?.documentMessage) {
        const filename = message.documentMessage.fileName || 'file';
        return path.extname(filename).slice(1) || 'bin';
    }
    if (message?.stickerMessage) return 'webp';
    return 'bin';
}

/**
 * Get media mimetype from message
 */
function getMediaMimetype(msg) {
    const message = msg.message;
    if (message?.imageMessage) return message.imageMessage.mimetype || 'image/jpeg';
    if (message?.videoMessage) return message.videoMessage.mimetype || 'video/mp4';
    if (message?.audioMessage) return message.audioMessage.mimetype || 'audio/ogg';
    if (message?.documentMessage) return message.documentMessage.mimetype || 'application/octet-stream';
    if (message?.stickerMessage) return 'image/webp';
    return 'application/octet-stream';
}

/**
 * Extract message content based on type
 */
function extractMessageContent(msg) {
    const message = msg.message;
    if (!message) return { type: 'unknown', content: null };

    // Handle different message types
    if (message.conversation) {
        return { type: 'text', content: message.conversation };
    }
    
    if (message.extendedTextMessage?.text) {
        return { 
            type: 'text', 
            content: message.extendedTextMessage.text,
            contextInfo: message.extendedTextMessage.contextInfo
        };
    }
    
    if (message.imageMessage) {
        return {
            type: 'image',
            content: message.imageMessage.caption || '',
            mediaInfo: {
                mimetype: message.imageMessage.mimetype,
                height: message.imageMessage.height,
                width: message.imageMessage.width,
                fileLength: message.imageMessage.fileLength
            }
        };
    }
    
    if (message.videoMessage) {
        return {
            type: 'video',
            content: message.videoMessage.caption || '',
            mediaInfo: {
                mimetype: message.videoMessage.mimetype,
                seconds: message.videoMessage.seconds,
                fileLength: message.videoMessage.fileLength
            }
        };
    }
    
    if (message.audioMessage) {
        return {
            type: message.audioMessage.ptt ? 'voice' : 'audio',
            content: '',
            mediaInfo: {
                mimetype: message.audioMessage.mimetype,
                seconds: message.audioMessage.seconds,
                fileLength: message.audioMessage.fileLength,
                ptt: message.audioMessage.ptt
            }
        };
    }
    
    if (message.documentMessage) {
        return {
            type: 'document',
            content: message.documentMessage.caption || '',
            mediaInfo: {
                filename: message.documentMessage.fileName,
                mimetype: message.documentMessage.mimetype,
                fileLength: message.documentMessage.fileLength
            }
        };
    }
    
    if (message.stickerMessage) {
        return {
            type: 'sticker',
            content: '',
            mediaInfo: {
                mimetype: 'image/webp',
                fileLength: message.stickerMessage.fileLength
            }
        };
    }
    
    if (message.locationMessage) {
        return {
            type: 'location',
            content: message.locationMessage.name || '',
            locationInfo: {
                latitude: message.locationMessage.degreesLatitude,
                longitude: message.locationMessage.degreesLongitude,
                name: message.locationMessage.name,
                address: message.locationMessage.address
            }
        };
    }
    
    if (message.contactMessage) {
        return {
            type: 'contact',
            content: message.contactMessage.vcard || '',
            contactInfo: {
                displayName: message.contactMessage.displayName,
                vcard: message.contactMessage.vcard
            }
        };
    }
    
    if (message.contactArrayMessage) {
        return {
            type: 'contacts',
            content: 'Multiple contacts',
            contactsCount: message.contactArrayMessage.contacts?.length || 0
        };
    }

    return { type: 'unknown', content: null };
}

/**
 * Handle incoming messages
 */
async function handleMessage(msg) {
    // Skip if duplicate
    if (isDuplicate(msg)) {
        log('debug', 'Skipping duplicate message', { id: msg.key.id });
        return;
    }

    // Skip messages from self
    if (msg.key.fromMe) {
        return;
    }

    const content = extractMessageContent(msg);
    
    // Download media if present
    let mediaData = null;
    if (['image', 'video', 'audio', 'voice', 'document', 'sticker'].includes(content.type)) {
        mediaData = await downloadMedia(msg);
    }

    // Build message event
    const messageEvent = {
        type: 'message',
        data: {
            id: msg.key.id,
            remoteJid: msg.key.remoteJid,
            fromMe: msg.key.fromMe,
            participant: msg.key.participant,
            timestamp: msg.messageTimestamp,
            pushName: msg.pushName,
            messageType: content.type,
            content: content.content,
            mediaInfo: content.mediaInfo || null,
            mediaData: mediaData,
            locationInfo: content.locationInfo || null,
            contactInfo: content.contactInfo || null,
            contextInfo: content.contextInfo || null,
            isGroup: msg.key.remoteJid.endsWith('@g.us')
        }
    };

    sendToRust(messageEvent);
}

/**
 * Create and configure WhatsApp socket
 */
async function createSocket() {
    const { state, saveCreds } = await useMultiFileAuthState(CONFIG.authDir);
    const { version, isLatest } = await fetchLatestBaileysVersion();
    
    log('info', `Using Baileys v${version.join('.')}, isLatest: ${isLatest}`);

    sock = makeWASocket({
        version,
        logger,
        printQRInTerminal: CONFIG.printQR,
        auth: {
            creds: state.creds,
            keys: makeCacheableSignalKeyStore(state.keys, logger),
        },
        generateHighQualityLinkPreview: true,
        syncFullHistory: false,
        markOnlineOnConnect: true,
        keepAliveIntervalMs: 30000,
        connectTimeoutMs: 60000,
        defaultQueryTimeoutMs: 60000,
        retryRequestDelayMs: 250,
        maxMsgRetryCount: 5,
        fireInitQueries: true,
        shouldIgnoreJid: (jid) => {
            // Ignore status broadcasts
            return jid === 'status@broadcast';
        },
        getMessage: async (key) => {
            // Return message for retry purposes
            return {
                conversation: 'Hello'
            };
        }
    });

    // Connection events
    sock.ev.on('connection.update', async (update) => {
        const { connection, lastDisconnect, qr } = update;
        
        if (qr) {
            connectionState = 'qr';
            log('info', 'QR code received, scan with WhatsApp');
            
            // Send QR to Rust
            sendToRust({
                type: 'qr',
                data: { qr }
            });
            
            if (CONFIG.printQR) {
                qrcode.generate(qr, { small: true });
            }
        }
        
        if (connection === 'close') {
            isConnected = false;
            connectionState = 'disconnected';
            const statusCode = lastDisconnect?.error?.output?.statusCode;
            const shouldReconnect = statusCode !== DisconnectReason.loggedOut;
            
            log('warn', 'Connection closed', { 
                statusCode, 
                reason: lastDisconnect?.error?.message,
                shouldReconnect 
            });
            
            sendToRust({
                type: 'disconnected',
                data: { 
                    reason: lastDisconnect?.error?.message || 'Unknown',
                    statusCode,
                    willReconnect: shouldReconnect
                }
            });
            
            if (shouldReconnect && reconnectAttempts < CONFIG.maxReconnectAttempts) {
                reconnectAttempts++;
                log('info', `Reconnecting... attempt ${reconnectAttempts}/${CONFIG.maxReconnectAttempts}`);
                
                sendToRust({
                    type: 'reconnecting',
                    data: { attempt: reconnectAttempts, maxAttempts: CONFIG.maxReconnectAttempts }
                });
                
                setTimeout(createSocket, CONFIG.reconnectInterval);
            } else if (!shouldReconnect) {
                log('error', 'Connection closed permanently (logged out)');
                sendToRust({
                    type: 'error',
                    data: { message: 'Logged out, please scan QR code again' }
                });
                // Clear auth state
                fs.rmSync(CONFIG.authDir, { recursive: true, force: true });
            }
        } else if (connection === 'open') {
            isConnected = true;
            connectionState = 'connected';
            reconnectAttempts = 0;
            
            log('info', 'Connected to WhatsApp', { 
                user: sock.user?.id,
                name: sock.user?.name 
            });
            
            sendToRust({
                type: 'connected',
                data: {
                    userId: sock.user?.id,
                    userName: sock.user?.name,
                    connectedAt: Date.now()
                }
            });
            
            // Process any queued messages
            while (messageQueue.length > 0) {
                const msg = messageQueue.shift();
                await sendMessage(msg);
            }
        } else if (connection === 'connecting') {
            connectionState = 'connecting';
            log('info', 'Connecting to WhatsApp...');
            
            sendToRust({
                type: 'connecting',
                data: {}
            });
        }
    });

    // Credentials update
    sock.ev.on('creds.update', saveCreds);

    // Message events
    sock.ev.on('messages.upsert', async (m) => {
        if (m.type === 'notify') {
            for (const msg of m.messages) {
                await handleMessage(msg);
            }
        }
    });

    // Message status updates (delivered, read, etc.)
    sock.ev.on('message-receipt.update', (updates) => {
        for (const update of updates) {
            sendToRust({
                type: 'message_receipt',
                data: {
                    messageId: update.key.id,
                    remoteJid: update.key.remoteJid,
                    receipt: update.receipt
                }
            });
        }
    });

    // Presence updates (typing, online, etc.)
    sock.ev.on('presence.update', (update) => {
        sendToRust({
            type: 'presence',
            data: update
        });
    });

    // Group participants update
    sock.ev.on('group-participants.update', (update) => {
        sendToRust({
            type: 'group_participants',
            data: update
        });
    });

    // Groups update
    sock.ev.on('groups.update', (updates) => {
        sendToRust({
            type: 'groups_update',
            data: updates
        });
    });

    return sock;
}

/**
 * Send a text message
 */
async function sendTextMessage(to, text, options = {}) {
    if (!isConnected) {
        throw new Error('Not connected to WhatsApp');
    }

    const jid = to.includes('@') ? to : `${to}@s.whatsapp.net`;
    
    const result = await sock.sendMessage(jid, {
        text: text,
        ...options
    });

    return {
        messageId: result.key.id,
        timestamp: result.messageTimestamp,
        remoteJid: result.key.remoteJid
    };
}

/**
 * Send an image message
 */
async function sendImageMessage(to, imagePath, caption = '', options = {}) {
    if (!isConnected) {
        throw new Error('Not connected to WhatsApp');
    }

    const jid = to.includes('@') ? to : `${to}@s.whatsapp.net`;
    const imageBuffer = fs.readFileSync(imagePath);
    
    const result = await sock.sendMessage(jid, {
        image: imageBuffer,
        caption: caption,
        ...options
    });

    return {
        messageId: result.key.id,
        timestamp: result.messageTimestamp,
        remoteJid: result.key.remoteJid
    };
}

/**
 * Send a video message
 */
async function sendVideoMessage(to, videoPath, caption = '', options = {}) {
    if (!isConnected) {
        throw new Error('Not connected to WhatsApp');
    }

    const jid = to.includes('@') ? to : `${to}@s.whatsapp.net`;
    const videoBuffer = fs.readFileSync(videoPath);
    
    const result = await sock.sendMessage(jid, {
        video: videoBuffer,
        caption: caption,
        ...options
    });

    return {
        messageId: result.key.id,
        timestamp: result.messageTimestamp,
        remoteJid: result.key.remoteJid
    };
}

/**
 * Send an audio message
 */
async function sendAudioMessage(to, audioPath, options = {}) {
    if (!isConnected) {
        throw new Error('Not connected to WhatsApp');
    }

    const jid = to.includes('@') ? to : `${to}@s.whatsapp.net`;
    const audioBuffer = fs.readFileSync(audioPath);
    
    const result = await sock.sendMessage(jid, {
        audio: audioBuffer,
        mimetype: 'audio/mp4',
        ptt: options.ptt || false, // Voice note if true
        ...options
    });

    return {
        messageId: result.key.id,
        timestamp: result.messageTimestamp,
        remoteJid: result.key.remoteJid
    };
}

/**
 * Send a document message
 */
async function sendDocumentMessage(to, documentPath, filename = '', caption = '', options = {}) {
    if (!isConnected) {
        throw new Error('Not connected to WhatsApp');
    }

    const jid = to.includes('@') ? to : `${to}@s.whatsapp.net`;
    const documentBuffer = fs.readFileSync(documentPath);
    
    const result = await sock.sendMessage(jid, {
        document: documentBuffer,
        fileName: filename || path.basename(documentPath),
        caption: caption,
        ...options
    });

    return {
        messageId: result.key.id,
        timestamp: result.messageTimestamp,
        remoteJid: result.key.remoteJid
    };
}

/**
 * Handle commands from Rust
 */
async function handleCommand(command) {
    try {
        switch (command.type) {
            case 'send_text':
                const textResult = await sendTextMessage(
                    command.data.to, 
                    command.data.text,
                    command.data.options || {}
                );
                sendToRust({
                    type: 'send_result',
                    id: command.id,
                    success: true,
                    data: textResult
                });
                break;

            case 'send_image':
                const imageResult = await sendImageMessage(
                    command.data.to,
                    command.data.imagePath,
                    command.data.caption || '',
                    command.data.options || {}
                );
                sendToRust({
                    type: 'send_result',
                    id: command.id,
                    success: true,
                    data: imageResult
                });
                break;

            case 'send_video':
                const videoResult = await sendVideoMessage(
                    command.data.to,
                    command.data.videoPath,
                    command.data.caption || '',
                    command.data.options || {}
                );
                sendToRust({
                    type: 'send_result',
                    id: command.id,
                    success: true,
                    data: videoResult
                });
                break;

            case 'send_audio':
                const audioResult = await sendAudioMessage(
                    command.data.to,
                    command.data.audioPath,
                    command.data.options || {}
                );
                sendToRust({
                    type: 'send_result',
                    id: command.id,
                    success: true,
                    data: audioResult
                });
                break;

            case 'send_document':
                const docResult = await sendDocumentMessage(
                    command.data.to,
                    command.data.documentPath,
                    command.data.filename,
                    command.data.caption || '',
                    command.data.options || {}
                );
                sendToRust({
                    type: 'send_result',
                    id: command.id,
                    success: true,
                    data: docResult
                });
                break;

            case 'get_status':
                sendToRust({
                    type: 'status',
                    id: command.id,
                    data: {
                        connected: isConnected,
                        state: connectionState,
                        user: sock?.user,
                        reconnectAttempts
                    }
                });
                break;

            case 'disconnect':
                await sock?.logout();
                sendToRust({
                    type: 'disconnected',
                    id: command.id,
                    data: { reason: 'manual' }
                });
                break;

            case 'get_groups':
                const groups = await sock?.groupFetchAllParticipating();
                sendToRust({
                    type: 'groups',
                    id: command.id,
                    data: groups || {}
                });
                break;

            case 'get_contacts':
                // Baileys doesn't store contacts, but we can return an empty list
                sendToRust({
                    type: 'contacts',
                    id: command.id,
                    data: []
                });
                break;

            default:
                sendToRust({
                    type: 'error',
                    id: command.id,
                    data: { message: `Unknown command: ${command.type}` }
                });
        }
    } catch (error) {
        log('error', 'Command failed', { command: command.type, error: error.message });
        sendToRust({
            type: 'error',
            id: command.id,
            data: { message: error.message, stack: error.stack }
        });
    }
}

/**
 * Process incoming line from stdin (from Rust)
 */
function processLine(line) {
    try {
        const command = JSON.parse(line);
        handleCommand(command);
    } catch (error) {
        log('error', 'Failed to parse command', { error: error.message, line });
        sendToRust({
            type: 'error',
            data: { message: 'Invalid JSON command', error: error.message }
        });
    }
}

/**
 * Main function
 */
async function main() {
    log('info', 'WhatsApp Baileys Bridge starting...', { config: CONFIG });
    
    // Setup stdin reader
    const readline = require('readline');
    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout,
        terminal: false
    });

    rl.on('line', processLine);

    // Handle process termination
    process.on('SIGINT', async () => {
        log('info', 'Shutting down...');
        await sock?.end();
        process.exit(0);
    });

    process.on('SIGTERM', async () => {
        log('info', 'Shutting down...');
        await sock?.end();
        process.exit(0);
    });

    // Create WhatsApp connection
    try {
        await createSocket();
    } catch (error) {
        log('error', 'Failed to create socket', { error: error.message });
        sendToRust({
            type: 'error',
            data: { message: 'Failed to initialize WhatsApp', error: error.message }
        });
    }
}

// Start the bridge
main().catch(error => {
    log('fatal', 'Bridge crashed', { error: error.message });
    process.exit(1);
});

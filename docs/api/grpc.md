# BeeBotOS gRPC API

> **高性能 RPC 接口**

---

## 概述

gRPC API 适用于：
- 高性能内部服务通信
- 流式数据传输
- 多语言 SDK 开发

---

## 基础信息

- **地址**: `grpc.beebotos.io:443`
- **协议**: gRPC over TLS (HTTP/2)
- **编码**: Protocol Buffers

## Proto 文件

```protobuf
// beebotos/api/v1/agent.proto
syntax = "proto3";

package beebotos.api.v1;

service AgentService {
  rpc CreateAgent(CreateAgentRequest) returns (Agent);
  rpc GetAgent(GetAgentRequest) returns (Agent);
  rpc ListAgents(ListAgentsRequest) returns (ListAgentsResponse);
  rpc UpdateAgent(UpdateAgentRequest) returns (Agent);
  rpc DeleteAgent(DeleteAgentRequest) returns (Empty);
  
  rpc StartAgent(StartAgentRequest) returns (Agent);
  rpc StopAgent(StopAgentRequest) returns (Agent);
  
  rpc Chat(ChatRequest) returns (ChatResponse);
  rpc StreamChat(stream ChatRequest) returns (stream ChatResponse);
}

message CreateAgentRequest {
  string name = 1;
  string description = 2;
  Personality personality = 3;
  repeated string capabilities = 4;
  Resources resources = 5;
}

message Agent {
  string agent_id = 1;
  string name = 2;
  string description = 3;
  string status = 4;
  Personality personality = 5;
  repeated string capabilities = 6;
  Resources resources = 7;
  string wallet_address = 8;
  int64 created_at = 9;
  int64 updated_at = 10;
}

message ChatRequest {
  string agent_id = 1;
  string message = 2;
  string session_id = 3;
  map<string, string> context = 4;
}

message ChatResponse {
  string response = 1;
  Emotion emotion = 2;
  string session_id = 3;
  string message_id = 4;
}

message Personality {
  PAD pad = 1;
  OCEAN ocean = 2;
}

message PAD {
  double pleasure = 1;
  double arousal = 2;
  double dominance = 3;
}

message OCEAN {
  double openness = 1;
  double conscientiousness = 2;
  double extraversion = 3;
  double agreeableness = 4;
  double neuroticism = 5;
}
```

---

## 客户端示例

### Go

```go
package main

import (
    "context"
    "log"
    
    pb "github.com/beebotos/api/go/v1"
    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials"
)

func main() {
    // 创建连接
    creds := credentials.NewClientTLSFromCert(nil, "")
    conn, err := grpc.Dial("grpc.beebotos.io:443", 
        grpc.WithTransportCredentials(creds))
    if err != nil {
        log.Fatal(err)
    }
    defer conn.Close()
    
    // 创建客户端
    client := pb.NewAgentServiceClient(conn)
    
    // 添加认证
    ctx := metadata.AppendToOutgoingContext(context.Background(),
        "authorization", "Bearer "+token)
    
    // 创建 Agent
    resp, err := client.CreateAgent(ctx, &pb.CreateAgentRequest{
        Name:        "MyAgent",
        Description: "Test agent",
        Capabilities: []string{"L3_NetworkOut"},
    })
    if err != nil {
        log.Fatal(err)
    }
    
    log.Printf("Agent created: %s", resp.AgentId)
}
```

### Python

```python
import grpc
from beebotos.api.v1 import agent_pb2, agent_pb2_grpc

# 创建通道
channel = grpc.secure_channel('grpc.beebotos.io:443', 
                              grpc.ssl_channel_credentials())

# 创建客户端
client = agent_pb2_grpc.AgentServiceStub(channel)

# 添加认证
metadata = [('authorization', 'Bearer ' + token)]

# 创建 Agent
response = client.CreateAgent(
    agent_pb2.CreateAgentRequest(
        name="MyAgent",
        description="Test agent",
        capabilities=["L3_NetworkOut"]
    ),
    metadata=metadata
)

print(f"Agent created: {response.agent_id}")
```

### Java

```java
import io.grpc.ManagedChannel;
import io.grpc.ManagedChannelBuilder;
import beebotos.api.v1.AgentServiceGrpc;
import beebotos.api.v1.AgentProto;

public class BeeBotOSClient {
    private final AgentServiceGrpc.AgentServiceBlockingStub stub;
    
    public BeeBotOSClient(String token) {
        ManagedChannel channel = ManagedChannelBuilder
            .forAddress("grpc.beebotos.io", 443)
            .useTransportSecurity()
            .build();
        
        stub = AgentServiceGrpc.newBlockingStub(channel)
            .withCallCredentials(new BearerToken(token));
    }
    
    public AgentProto.Agent createAgent(String name) {
        AgentProto.CreateAgentRequest request = 
            AgentProto.CreateAgentRequest.newBuilder()
                .setName(name)
                .addCapabilities("L3_NetworkOut")
                .build();
        
        return stub.createAgent(request);
    }
}
```

---

## 流式 API

### 双向流式对话

```protobuf
rpc StreamChat(stream ChatRequest) returns (stream ChatResponse);
```

**Go 示例**:

```go
stream, err := client.StreamChat(ctx)
if err != nil {
    log.Fatal(err)
}

// 发送消息
go func() {
    for {
        msg := <-inputCh
        stream.Send(&pb.ChatRequest{
            AgentId: agentId,
            Message: msg,
        })
    }
}()

// 接收响应
for {
    resp, err := stream.Recv()
    if err == io.EOF {
        break
    }
    if err != nil {
        log.Fatal(err)
    }
    fmt.Println(resp.Response)
}
```

---

**文档版本**: v1.0.0

#!/usr/bin/env python3
"""
BeeBotOS Basic Agent Example
Demonstrates how to create and interact with a BeeBotOS agent using Python.
"""

import asyncio
import json
from typing import Dict, Any

class BeeBotOSAgent:
    """Simple BeeBotOS Agent implementation."""
    
    def __init__(self, name: str, config: Dict[str, Any]):
        self.name = name
        self.config = config
        self.state = "idle"
        self.memory = {}
        
    async def initialize(self):
        """Initialize the agent."""
        print(f"Initializing agent: {self.name}")
        self.state = "ready"
        return True
    
    async def process_task(self, task: Dict[str, Any]) -> Dict[str, Any]:
        """Process a task."""
        print(f"Processing task: {task.get('type')}")
        
        task_type = task.get('type')
        
        if task_type == 'chat':
            return await self._handle_chat(task)
        elif task_type == 'data_analysis':
            return await self._handle_analysis(task)
        else:
            return {'status': 'error', 'message': f'Unknown task type: {task_type}'}
    
    async def _handle_chat(self, task: Dict[str, Any]) -> Dict[str, Any]:
        """Handle chat task."""
        message = task.get('input', '')
        response = f"BeeBotOS Agent {self.name} received: {message}"
        
        return {
            'status': 'success',
            'response': response,
            'agent': self.name
        }
    
    async def _handle_analysis(self, task: Dict[str, Any]) -> Dict[str, Any]:
        """Handle data analysis task."""
        data = task.get('data', [])
        
        result = {
            'count': len(data),
            'sum': sum(data) if data else 0,
            'average': sum(data) / len(data) if data else 0
        }
        
        return {
            'status': 'success',
            'analysis': result
        }
    
    def store_memory(self, key: str, value: Any):
        """Store something in agent memory."""
        self.memory[key] = value
        
    def recall_memory(self, key: str) -> Any:
        """Recall from agent memory."""
        return self.memory.get(key)


async def main():
    """Main example function."""
    
    # Create agent configuration
    config = {
        'model': 'gpt-4',
        'capabilities': ['chat', 'data_analysis'],
        'memory_enabled': True
    }
    
    # Create and initialize agent
    agent = BeeBotOSAgent("MyFirstAgent", config)
    await agent.initialize()
    
    # Example 1: Chat task
    chat_task = {
        'type': 'chat',
        'input': 'Hello, how are you?'
    }
    
    result = await agent.process_task(chat_task)
    print(f"Chat result: {json.dumps(result, indent=2)}")
    
    # Example 2: Data analysis task
    analysis_task = {
        'type': 'data_analysis',
        'data': [10, 20, 30, 40, 50]
    }
    
    result = await agent.process_task(analysis_task)
    print(f"Analysis result: {json.dumps(result, indent=2)}")
    
    # Example 3: Memory usage
    agent.store_memory('user_preference', 'dark_mode')
    preference = agent.recall_memory('user_preference')
    print(f"Recalled preference: {preference}")


if __name__ == "__main__":
    asyncio.run(main())

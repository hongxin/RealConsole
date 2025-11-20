# Gemini API Format vs OpenAI Format

**Date**: 2025-01-20
**Purpose**: Document API format differences for Gemini integration

## Key Differences Summary

| Aspect | OpenAI/Deepseek | Gemini |
|--------|-----------------|--------|
| **Endpoint** | `POST /v1/chat/completions` | `POST /v1beta/models/{model}:generateContent` |
| **Auth** | Header: `Authorization: Bearer {key}` | Query param: `?key={key}` |
| **Model** | In request body: `{"model": "gpt-4"}` | In URL path: `models/gemini-pro` |
| **Messages** | `messages: [{role, content}]` | `contents: [{role, parts}]` |
| **System** | `{role: "system", content: "..."}` | `systemInstruction: {parts: {text}}` |
| **User Role** | `"user"` | `"user"` ✅ Same |
| **Assistant Role** | `"assistant"` | `"model"` ⚠️ Different |
| **Response** | `choices[].message.content` | `candidates[].content.parts[].text` |
| **Tools** | `tools: [{type: "function", function: {...}}]` | `tools: [{functionDeclarations: [...]}]` |
| **Tool Response** | `tool_calls: [{id, function: {name, arguments}}]` | `parts: [{functionCall: {name, args}}]` |

## Request Format Examples

### OpenAI Format
```json
{
  "model": "gpt-4",
  "messages": [
    {"role": "system", "content": "You are helpful"},
    {"role": "user", "content": "Hello"}
  ]
}
```

### Gemini Format
```json
{
  "contents": [
    {
      "role": "user",
      "parts": [{"text": "Hello"}]
    }
  ],
  "systemInstruction": {
    "parts": {"text": "You are helpful"}
  }
}
```

## Response Format Examples

### OpenAI Response
```json
{
  "choices": [{
    "message": {
      "role": "assistant",
      "content": "Hi there!"
    }
  }]
}
```

### Gemini Response
```json
{
  "candidates": [{
    "content": {
      "role": "model",
      "parts": [{"text": "Hi there!"}]
    }
  }]
}
```

## Function Calling Differences

### OpenAI Tools Format
```json
{
  "tools": [{
    "type": "function",
    "function": {
      "name": "get_weather",
      "description": "Get weather",
      "parameters": {
        "type": "object",
        "properties": {
          "location": {"type": "string"}
        }
      }
    }
  }]
}
```

### Gemini Tools Format
```json
{
  "tools": [{
    "functionDeclarations": [{
      "name": "get_weather",
      "description": "Get weather",
      "parameters": {
        "type": "object",
        "properties": {
          "location": {"type": "string"}
        }
      }
    }]
  }]
}
```

## Implementation Strategy

1. **Message Conversion**: Transform OpenAI `Message` format to Gemini `contents`
2. **Role Mapping**: Map `assistant` → `model`, extract `system` messages
3. **Response Parsing**: Extract text from `candidates[].content.parts[]`
4. **Tool Schema Transform**: Wrap function schemas in `functionDeclarations`
5. **Tool Response Parsing**: Parse `functionCall` from `parts` array

## References

- Gemini API Docs: https://ai.google.dev/gemini-api/docs
- REST API Reference: https://ai.google.dev/api/generate-content

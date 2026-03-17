use anyhow::{Context, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use std::fs;

use super::models::{AuthInfo, AuthMode, PlanType};

/// 从 auth.json 文件解析认证信息
pub fn parse_auth_info(auth_path: &str) -> Result<AuthInfo> {
    let data = fs::read_to_string(auth_path)
        .with_context(|| format!("无法读取认证文件: {}", auth_path))?;

    let root: serde_json::Value =
        serde_json::from_str(&data).with_context(|| "认证文件 JSON 解析失败")?;

    let obj = match root.as_object() {
        Some(o) => o,
        None => {
            return Ok(AuthInfo {
                email: None,
                plan: None,
                auth_mode: AuthMode::Chatgpt,
            })
        }
    };

    // 检测 API Key 模式
    if let Some(key_val) = obj.get("OPENAI_API_KEY") {
        if let Some(s) = key_val.as_str() {
            if !s.is_empty() {
                return Ok(AuthInfo {
                    email: None,
                    plan: None,
                    auth_mode: AuthMode::Apikey,
                });
            }
        }
    }

    // 检测 ChatGPT 模式（JWT Token）
    if let Some(tokens_val) = obj.get("tokens") {
        if let Some(tokens_obj) = tokens_val.as_object() {
            if let Some(id_tok) = tokens_obj.get("id_token") {
                if let Some(jwt) = id_tok.as_str() {
                    let payload = decode_jwt_payload(jwt)?;
                    let claims: serde_json::Value = serde_json::from_str(&payload)
                        .with_context(|| "JWT payload JSON 解析失败")?;

                    let mut email: Option<String> = None;
                    let mut plan: Option<PlanType> = None;

                    if let Some(claims_obj) = claims.as_object() {
                        // 提取邮箱并转小写
                        if let Some(e) = claims_obj.get("email") {
                            if let Some(s) = e.as_str() {
                                email = Some(s.to_lowercase());
                            }
                        }

                        // 提取计划类型
                        if let Some(auth_obj) = claims_obj.get("https://api.openai.com/auth") {
                            if let Some(aobj) = auth_obj.as_object() {
                                if let Some(pt) = aobj.get("chatgpt_plan_type") {
                                    if let Some(s) = pt.as_str() {
                                        plan = Some(PlanType::from_str_loose(s));
                                    }
                                }
                            }
                        }
                    }

                    return Ok(AuthInfo {
                        email,
                        plan,
                        auth_mode: AuthMode::Chatgpt,
                    });
                }
            }
        }
    }

    Ok(AuthInfo {
        email: None,
        plan: None,
        auth_mode: AuthMode::Chatgpt,
    })
}

/// 解码 JWT 的 payload 部分（Base64URL 无填充）
pub fn decode_jwt_payload(jwt: &str) -> Result<String> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        anyhow::bail!("无效的 JWT 格式：需要 3 个部分，实际 {} 个", parts.len());
    }

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .with_context(|| "JWT payload Base64 解码失败")?;

    String::from_utf8(payload_bytes).with_context(|| "JWT payload 不是有效的 UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_jwt_payload() {
        // 构造一个简单的 JWT：header.payload.signature
        // payload = {"email":"test@example.com","https://api.openai.com/auth":{"chatgpt_plan_type":"plus"}}
        let payload_json = r#"{"email":"test@example.com","https://api.openai.com/auth":{"chatgpt_plan_type":"plus"}}"#;
        let encoded = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());
        let jwt = format!("eyJ0eXAiOi.{}.signature", encoded);

        let decoded = decode_jwt_payload(&jwt).unwrap();
        assert_eq!(decoded, payload_json);
    }

    #[test]
    fn test_decode_jwt_payload_invalid() {
        assert!(decode_jwt_payload("not.a").is_err());
        assert!(decode_jwt_payload("only_one_part").is_err());
    }
}

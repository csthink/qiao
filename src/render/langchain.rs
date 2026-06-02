//! LangChain 代码片段渲染(Python,OpenAI 兼容,spec S5)。
//!
//! 目标 provider 多为 OpenAI 兼容,v1 统一产出 `ChatOpenAI` 形态。`base_url` 必填;
//! 有 chat 角色才出 `model=` 行(缺则省略,由 ChatOpenAI 取默认)。

use anyhow::Result;
use zeroize::Zeroizing;

use super::require;
use crate::model::Provider;
use crate::secret::Secret;

/// 渲染 LangChain `ChatOpenAI` 片段。返回值含明文 key,故用 [`Secret`] 包裹。
pub fn render(id: &str, p: &Provider, key: &Secret) -> Result<Secret> {
    let base_url = require(p.base_url.as_deref(), id, "base_url")?;

    let mut out = Zeroizing::new(String::new());
    out.push_str("from langchain_openai import ChatOpenAI\n");
    out.push_str("llm = ChatOpenAI(\n");
    {
        let mut kwarg = |name: &str, value: &str| {
            out.push_str("    ");
            out.push_str(name);
            out.push_str("=\"");
            out.push_str(value);
            out.push_str("\",\n");
        };
        kwarg("base_url", base_url);
        if let Some(chat) = p.models.chat() {
            kwarg("model", chat);
        }
        kwarg("api_key", key.as_str());
    }
    out.push_str(")\n");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Models;

    fn provider(base_url: Option<&str>, roles: &[(&str, &str)]) -> Provider {
        let mut models = Models::default();
        for (k, v) in roles {
            models.roles.insert(k.to_string(), v.to_string());
        }
        Provider {
            base_url: base_url.map(str::to_string),
            models,
            ..Default::default()
        }
    }

    fn key() -> Secret {
        Zeroizing::new("sk-TEST".to_string())
    }

    #[test]
    fn openrouter_matches_s5_exactly() {
        let p = provider(
            Some("https://openrouter.ai/api/v1"),
            &[("chat", "openai/gpt-5.5"), ("embedding", "baai/bge-m3")],
        );
        let out = render("openrouter", &p, &key()).unwrap();
        assert_eq!(
            &*out,
            "from langchain_openai import ChatOpenAI\n\
             llm = ChatOpenAI(\n\
             \x20   base_url=\"https://openrouter.ai/api/v1\",\n\
             \x20   model=\"openai/gpt-5.5\",\n\
             \x20   api_key=\"sk-TEST\",\n\
             )\n"
        );
    }

    #[test]
    fn omits_model_when_chat_absent() {
        let p = provider(Some("https://x/v1"), &[]);
        let out = render("x", &p, &key()).unwrap();
        assert_eq!(
            &*out,
            "from langchain_openai import ChatOpenAI\n\
             llm = ChatOpenAI(\n\
             \x20   base_url=\"https://x/v1\",\n\
             \x20   api_key=\"sk-TEST\",\n\
             )\n"
        );
    }

    #[test]
    fn errors_when_base_url_missing() {
        let p = provider(None, &[("chat", "m")]);
        let e = render("aliyun_bailian", &p, &key()).unwrap_err().to_string();
        assert!(e.contains("aliyun_bailian") && e.contains("base_url"));
    }
}

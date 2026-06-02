//! `.env` 片段渲染(spec S5)。
//!
//! 变量名 = `env_prefix` + 固定后缀:`_API_KEY` / `_BASE_URL` / `_MODEL` / `_EMBEDDING_MODEL`。
//! 缺某模型角色则省略对应行;`env_prefix` / `base_url` 缺失报错。

use anyhow::Result;
use zeroize::Zeroizing;

use super::require;
use crate::model::Provider;
use crate::secret::Secret;

/// 渲染 `.env` 片段。返回值含明文 key,故用 [`Secret`] 包裹。
pub fn render(id: &str, p: &Provider, key: &Secret) -> Result<Secret> {
    let prefix = require(p.env_prefix.as_deref(), id, "env_prefix")?;
    let base_url = require(p.base_url.as_deref(), id, "base_url")?;

    // 直接拼进受控缓冲:明文 key 只在此一处落点。块作用域让闭包先析构,再 move 出 out。
    let mut out = Zeroizing::new(String::new());
    {
        let mut line = |k: &str, v: &str| {
            out.push_str(prefix);
            out.push_str(k);
            out.push('=');
            out.push_str(v);
            out.push('\n');
        };
        line("_API_KEY", key.as_str());
        line("_BASE_URL", base_url);
        if let Some(chat) = p.models.chat() {
            line("_MODEL", chat);
        }
        if let Some(embedding) = p.models.embedding() {
            line("_EMBEDDING_MODEL", embedding);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Models;

    fn provider(prefix: Option<&str>, base_url: Option<&str>, roles: &[(&str, &str)]) -> Provider {
        let mut models = Models::default();
        for (k, v) in roles {
            models.roles.insert(k.to_string(), v.to_string());
        }
        Provider {
            env_prefix: prefix.map(str::to_string),
            base_url: base_url.map(str::to_string),
            models,
            ..Default::default()
        }
    }

    fn key() -> Secret {
        // 占位串,非真实 key。
        Zeroizing::new("sk-TEST".to_string())
    }

    #[test]
    fn openrouter_matches_s5_exactly() {
        let p = provider(
            Some("OPENROUTER"),
            Some("https://openrouter.ai/api/v1"),
            &[("chat", "openai/gpt-5.5"), ("embedding", "baai/bge-m3")],
        );
        let out = render("openrouter", &p, &key()).unwrap();
        assert_eq!(
            &*out,
            "OPENROUTER_API_KEY=sk-TEST\n\
             OPENROUTER_BASE_URL=https://openrouter.ai/api/v1\n\
             OPENROUTER_MODEL=openai/gpt-5.5\n\
             OPENROUTER_EMBEDDING_MODEL=baai/bge-m3\n"
        );
    }

    #[test]
    fn omits_embedding_line_when_absent() {
        // deepseek 无 embedding(DoD:无 embedding 时省略对应行)。
        let p = provider(
            Some("DEEPSEEK"),
            Some("https://api.deepseek.com/v1"),
            &[("chat", "deepseek-v4-pro")],
        );
        let out = render("deepseek", &p, &key()).unwrap();
        assert_eq!(
            &*out,
            "DEEPSEEK_API_KEY=sk-TEST\n\
             DEEPSEEK_BASE_URL=https://api.deepseek.com/v1\n\
             DEEPSEEK_MODEL=deepseek-v4-pro\n"
        );
        assert!(!out.contains("EMBEDDING"));
    }

    #[test]
    fn omits_model_line_when_chat_absent() {
        let p = provider(Some("X"), Some("https://x/v1"), &[]);
        let out = render("x", &p, &key()).unwrap();
        assert_eq!(&*out, "X_API_KEY=sk-TEST\nX_BASE_URL=https://x/v1\n");
    }

    #[test]
    fn errors_name_provider_and_field() {
        let no_prefix = provider(None, Some("https://x/v1"), &[]);
        let e = render("siliconflow", &no_prefix, &key()).unwrap_err().to_string();
        assert!(e.contains("siliconflow") && e.contains("env_prefix"));

        let no_url = provider(Some("X"), None, &[]);
        let e = render("siliconflow", &no_url, &key()).unwrap_err().to_string();
        assert!(e.contains("base_url"));
    }
}

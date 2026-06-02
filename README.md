# qiao(桥)

> 桥接多家 LLM provider 的配置中心。从 keychain / Bitwarden 取出 API key,
> 一键生成可直接粘贴的 `.env` 片段或 LangChain 代码片段——免去每次 google provider、
> 翻文档找 base_url、再去后台找 key 的重复劳动。

**站在巨人肩上,不重复造轮子**:provider/模型目录复用 [models.dev](https://models.dev),
密钥存储复用操作系统 keychain 与你已有的 Bitwarden/Vaultwarden,qiao 只写很薄的整合层。

> ⚠️ **当前状态:设计完成,实现进行中。** 本 README 为初始版;安装与用法部分将在实现完成(tasks T7)后回填。

---

## 它解决什么

用 LangChain 写 agent 时需要频繁切换不同 provider 测试模型,痛点有三:

1. 死知识反复查——每家的 base_url、模型 ID 命名规则、embedding 模型名,每次都要重新搜。
2. 密钥散落不安全——API key 记在明文笔记里,有泄露和丢失风险。
3. 切换繁琐——想用某个模型时要手动拼一整套配置。

qiao 把这些收敛成几条命令:列出 provider、查看某家配置、取 key 拼出 `.env` 或代码片段。

## 核心原则:机密与配置分家

| 类别 | 内容 | 存放 | 是否落盘 |
|---|---|---|---|
| 机密 | API key | keychain / `bw` | 否(只存引用) |
| 非机密配置 | base_url、模型 ID、embedding、env 变量名 | 配置目录(快照 + models.dev + 覆盖) | 是(明文,可提交/分享) |

qiao 自身**不持有、不落盘**任何密钥。

## 命名约定速查

**凭证引用 URI**(配置里只存引用,绝不存 key):
```
<backend>:<locator>[#profile]
keychain:openrouter            # 默认 profile
keychain:openrouter#work       # 多账号
bw:item/OpenRouter API Key     # Bitwarden 按条目名
bw:id/2a16-445b-...            # Bitwarden 按条目 id(更稳)
env:OPENROUTER_API_KEY         # 环境变量兜底
```

**keychain 布局**:`service = "dev.mars.qiao"`,`account = "<provider>[#profile]"`,一个条目一个 key。

> 注:Bitwarden 走 **`bw`(Password Manager CLI)**,可连自托管 Vaultwarden;
> **不用 `bws`(Secrets Manager)**——它非开源、Vaultwarden 不支持。

## 设计文档

完整规格在 [`docs/`](./docs/),按 OpenSpec 四文档结构组织:

| 文档 | 内容 |
|---|---|
| [proposal.md](./docs/proposal.md) | 动机、范围、已锁定决策 |
| [spec.md](./docs/spec.md) | 可测试的行为契约(命令、引用语法、schema、输出格式) |
| [design.md](./docs/design.md) | Rust 架构、crate 结构、SecretStore trait |
| [tasks.md](./docs/tasks.md) | T0–T7 实现任务拆解(单文件单 owner) |
| [evaluator-handoff.md](./docs/evaluator-handoff.md) | 接力协议:generator/evaluator 分工与验收门 |

provider 内置快照:[`snapshot/providers.snapshot.toml`](./snapshot/providers.snapshot.toml)(运行时资源)。

## 开发方式:Planner / Generator / Evaluator 接力

本项目用多角色接力开发,文档是唯一事实来源:

- **Generator**(Claude Code)按 `tasks.md` 顺序写代码,每完成一任务写 `.relay/Tn.status.md`。
- **Evaluator**(独立 Claude 会话)对照规格评审,产出 `.relay/Tn.review.md`,PASS 才放行下一任务。
- **Planner / 仲裁**(维护者)传递 status/review、拍板范围变更、做最终放行。

详见 [evaluator-handoff.md](./docs/evaluator-handoff.md)。

## 技术栈与范围(v1)

- 语言:Rust(单静态二进制),平台:**仅 macOS**
- 密钥后端:`keychain`(默认)/ `bw` / `env`
- 目录:models.dev 拉取 + 内置快照兜底 + 用户本地覆盖
- 模型角色:`chat` + `embedding`(schema 预留扩展)
- 输出:`.env` 片段 + LangChain 代码片段

**v1 不做**(数据模型为其预留):机密注入子进程(`run --`)、Linux/headless、Vault 后端、GUI、签名公证。

## 安装与用法

> 🚧 待实现完成(tasks T7)后回填:`cargo install` / Homebrew 安装、`qiao` 各子命令示例、
> `bw` 连 Vaultwarden 的前置配置。

## License

[MIT](./LICENSE) © 2026 mars

开源,无营收目标,旨在解放程序员的重复劳动。
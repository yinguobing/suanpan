# 版本发布检查清单

本文档记录了发布 suanpan 新版本的标准流程。

## 发布前检查

- [ ] 所有测试通过 (`cargo test`)
- [ ] 代码格式化正确 (`cargo fmt --check`)
- [ ] Clippy 无警告 (`cargo clippy -- -D warnings`)
- [ ] 本地构建成功 (`cargo build --release`)
- [ ] README 文档已更新（如需要）

## 版本发布步骤

### 1. 更新版本号

编辑 `Cargo.toml` 更新版本号：

```toml
[package]
version = "0.2.0"  # 更新此版本号
```

版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)：
- `MAJOR`: 不兼容的 API 修改
- `MINOR`: 向下兼容的功能添加
- `PATCH`: 向下兼容的问题修复

### 2. 更新 CHANGELOG.md

在 `CHANGELOG.md` 顶部添加新版本记录：

```markdown
## [0.2.0] - 2026-04-15

### Added
- 新增功能 A
- 新增功能 B

### Changed
- 改进功能 C

### Fixed
- 修复问题 D
```

### 3. 提交更改

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore(release): 准备 v0.2.0 发布"
```

### 4. 创建标签

```bash
git tag -a v0.2.0 -m "Release v0.2.0"
```

### 5. 推送到 GitHub

```bash
git push origin main
git push origin v0.2.0
```

推送标签后会自动触发 [Release Workflow](.github/workflows/release.yml)，构建多平台二进制文件并创建 GitHub Release。

### 6. 验证发布

- [ ] GitHub Actions 工作流成功完成
- [ ] GitHub Release 页面已创建
- [ ] 所有平台二进制文件已上传
- [ ] 校验和文件已上传

访问 Release 页面：
```
https://github.com/yinguobing/suanpan/releases/tag/v0.2.0
```

### 7. 测试安装

测试一键安装脚本：

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/yinguobing/suanpan/main/scripts/install.sh | sh
```

测试 cargo install：

```bash
cargo install --git https://github.com/yinguobing/suanpan --tag v0.2.0
```

## 版本号规则

| 版本格式 | 含义 | 示例 |
|---------|------|------|
| `v0.x.x` | 初始开发阶段 | `v0.1.0`, `v0.2.3` |
| `vx.x.x-alpha.n` | 预发布 - 内测版 | `v1.0.0-alpha.1` |
| `vx.x.x-beta.n` | 预发布 - 公测版 | `v1.0.0-beta.2` |
| `vx.x.x-rc.n` | 预发布 - 候选版 | `v1.0.0-rc.1` |
| `vx.x.x` | 正式版 | `v1.0.0` |

预发布版本在 GitHub Release 中会自动标记为 "Pre-release"。

## 紧急修复流程

如需紧急修复生产版本：

1. 从对应版本的 tag 创建修复分支：
   ```bash
   git checkout -b hotfix/v0.1.1 v0.1.0
   ```

2. 修复问题并提交

3. 更新版本号为补丁版本 (`0.1.1`)

4. 打标签并推送：
   ```bash
   git tag -a v0.1.1 -m "Hotfix v0.1.1"
   git push origin hotfix/v0.1.1
   git push origin v0.1.1
   ```

## 相关链接

- [GitHub Releases](https://github.com/yinguobing/suanpan/releases)
- [GitHub Actions](https://github.com/yinguobing/suanpan/actions)
- [CHANGELOG.md](./CHANGELOG.md)

# 2026-11-09 解除归档并恢复可写状态

## 背景

仓库在 2026-10 被归档为只读，`d938396` 在 README 顶部写明「项目状态：已归档」。
此后本地仍有两个未推送的提交：`10dd89e`（SA:Z MAPQ、FM 索引防御、限带
semiglobal）与 `f2f8087`（种子链 DP 前驱窗口剪枝）。

## 处理

- 用 `gh repo unarchive open-genomics/bwa-rust` 解除归档，`git push` 补齐
  `d938396..f2f8087`，本地与远端一致。
- 远端 CI（MSRV、fmt/clippy/tests）与 Security Audit（cargo audit）在 `f2f8087`
  上均通过。
- 仓库保持**可写**，因此 README 的状态说明改为「暂停功能开发（2026-10 起）」：
  新功能仍由 minibwa-rust 承接，本仓库只接受缺陷修复与文档改进，避免公开状态
  与文档口径互相矛盾。

## 后果

- 仓库不再是只读快照，任何后续推送都需要显式发起（不会自动产生新提交）。
- 若之后要恢复归档，先确认 README 状态说明是否需要同步改回。

# 页脚备案与年份配置设计

**目标**

让页脚中的起始年份和备案号来自 `dand.toml`，结束年份在渲染时按当前年份生成；域名文案继续保持模板里的固定值。

## 现状

- `template/layout/copyright.html` 直接写死了 `2020 - 2023` 和 `沪ICP备20002980号`
- `src/site.rs` 的 `SiteConf` 目前只有 `title`、`home`、`description`
- 各页面渲染器都会把 `site` 注入 Tera，但没有统一的页脚上下文

## 决策

### 配置结构

在 `dand.toml` 中新增分组：

```toml
[footer]
start_year = 2020
beian = "沪ICP备20002980号"
```

原因：

- 页脚配置集中管理，避免污染顶层字段
- 后续若增加页脚链接、署名等字段，结构更自然

### 渲染规则

- `start_year` 来自配置
- `end_year` 在渲染时取当前本地年份
- 展示文案规则：
  - 若 `start_year < current_year`，显示 `start_year - current_year`
  - 若 `start_year == current_year`，只显示单年
- 备案号原样输出到模板

### 范围控制

本次只处理：

- 起始年份可配置
- 备案号可配置
- 结束年份动态生成

不处理：

- `sunnysab.cn` 域名文案配置化
- 页脚整体结构调整

## 实现方案

1. 在 `src/site.rs` 中为 `SiteConf` 增加 `footer: FooterConf`
2. 新增可序列化的页脚渲染数据结构，负责把配置转换成模板可直接消费的字符串
3. 在 `src/index.rs`、`src/post/render.rs` 等渲染入口统一注入 `footer`
4. 修改 `template/layout/copyright.html`，改为读取 `footer.year_label` 和 `footer.beian`
5. 在 `tests/build_cli.rs` 增加回归测试，覆盖：
   - `[footer]` 配置被正确读取
   - 输出 HTML 包含动态当前年份
   - 输出 HTML 包含配置的备案号

## 风险与兼容性

- 这是 `dand.toml` 的新增必填字段；现有测试和示例配置需要同步更新
- 动态年份依赖运行环境本地时间；测试只断言当前年份文本，不依赖具体日期

## 验证

- 先写 CLI 集成测试并确认失败
- 实现最小代码让测试通过
- 最后运行相关测试与 `cargo test`

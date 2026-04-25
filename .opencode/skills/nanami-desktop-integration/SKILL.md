---
name: nanami-desktop-integration
description: 当实现托盘、KDE/Wayland 集成、通知、截图、剪贴板、全局快捷键、开机自启、窗口置顶或跨平台桌面能力时使用。
---
# Nanami Desktop Integration Skill

## 原则

1. Linux 优先支持 KDE Wayland，X11 作为回退。
2. 截图、剪贴板、全局快捷键等能力必须经过权限层。
3. Wayland 下不要依赖 X11-only hack。
4. 优先使用 xdg-desktop-portal / Qt 标准能力。
5. 桌面能力必须跨平台抽象，不要把 KDE 逻辑散落在业务层。
6. 系统通知不得泄露敏感内容，默认只显示摘要。

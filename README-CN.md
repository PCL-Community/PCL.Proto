# PCL.Proto

<p align="center">
  <img src="./public/PCL.Proto.svg" width="200" alt="PCL.Proto" />
</p>

随着PCL的分支版本竞相启动，UI的还原成为了一大困扰众开发者的难题。PCL.Proto 应运而生。

![screenshot](./screenshot.png)

本项目以[PCL2（龙腾猫跃）](https://github.com/Hex-Dragon/PCL2)和[PCL2-CE](https://github.com/PCL-Community/PCL2-CE)为蓝本。旨在为各PCL分支版本提供一个标准化的原型样本。该仓库使用 Vue3 搭建，如果你的仓库使用 Webview 作为前端，则可以直接引用该项目。

你可以前往 [PCL.Proto](https://amagicpear.top/PCL.Proto/) 在线查看本项目。

## 原型

如果你的 PCL 分支版本并非基于 Web 技术，则直接使用本仓库可能并不显得那么方便。那么，你可以前往 [即时设计 - 「PCL.Prototype」](https://js.design/f/QVPQRY?p=zX2rcVk6Cy&mode=design)查看具体的应用原型，并切图导出或参考其生成的样式代码。

如果你想协助修改原型设计文件，欢迎[点击此链接](https://js.design/ti?c=tS-6qs0WDQJ3H4)加入团队。

## 项目配置

### 前置环境

- 请确保你的设备上已安装`nodejs`或`bun`。如果你使用`bun`作为包管理器，可以将下面出现的所有`pnpm`命令替换为`bun`。
- 设备应已安装可用的`rust`工具链。对于 Windows，需要使用 MSVC 版本的工具链。并确保你已经在 Visual Studio 安装器中安装了 C++ 开发工具中的 MSVC 套件以及 Windows SDK；对于 macOS，需要确保已安装 Xcode。
- 对于 Windows 平台，还需要安装 [Npcap](https://npcap.com/#download)。


> [!NOTE]
> 如有疑惑，可参考 [前置要求 | Tauri](https://tauri.app/zh-cn/start/prerequisites/)。

### 项目依赖

在克隆本项目后，你需要运行以下命令初始化子模块：

```sh
git submodule update --init --recursive
```

目前，项目已成为 Tauri + Vue3 + Vite 结合项目。在首次运行本项目之前，请先运行下面的命令安装前端依赖：

```sh
pnpm install
```

后端依赖会由 cargo 自动安装，无需手动管理。

然后，使用 `pnpm run tauri icon`自动生成图标。接下来就可以开始正式的开发和构建了。

> [!WARNING]
> 本仓库已排除图标文件。因此如果你在克隆仓库后不进行此步，icons文件夹内为空的话，在 **Tauri 应用启动时会报错**！

### 开发时热重载

```sh
pnpm dev
```

### 构建 Tauri 应用

如果你的设备上只安装了`bun`但没有安装`nodejs`，那么请将`src-tauri/tauri.conf.json`中的`build/beforeBuildCommand`内容从`pnpm run build`更改为`bun run build-only`。

```sh
pnpm run tauri build
```

## 项目打包求助！

我想把这个项目做成 Vue 组件的 npm 包，但是我不会做，如果有大佬会做组件库的话，欢迎联系我！！！

## 鸣谢

### 直接引用项目

> 如果我看得更远，那是因为我站在巨人的肩膀上。
> —— Sir Isaac Newton

[Vue.js](https://github.com/vuejs/core)

[TypeScript](https://github.com/microsoft/TypeScript)

[Pug](https://github.com/pugjs/pug)

[Vite](https://github.com/vitejs/vite)

[Vue Router](https://github.com/vuejs/vue-router-next)

[xml-js](https://www.npmjs.com/package/xml-js)

[Pinia](https://pinia.vuejs.org/)

[skinview3d](https://github.com/bs-community/skinview3d)

[Bun](https://bun.com/)

[Rust Programming Language](https://www.rust-lang.org/)

[Tauri](https://tauri.app/)

### 参考项目实现

[Plain Craft Launcher](https://github.com/Meloong-Git/PCL)

[PCL2-CE](https://github.com/PCL-Community/PCL2-CE)

[PCL.Neo](https://github.com/PCL-Community/PCL.Neo)

[xphost008/MMCLL](https://github.com/xphost008/MMCLL)

[Steve-xmh/scl](https://github.com/Steve-xmh/scl)

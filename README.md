# PCL.Proto

<p align="center">
  <img src="./public/PCL.Proto.svg" width="200" alt="PCL.Proto" />
</p>

随着PCL的分支版本竞相启动，UI的还原成为了一大困扰众开发者的难题。PCL.Proto 应运而生。

![screenshot](./screenshot.png)

本项目以[PCL2（龙腾猫跃）](https://github.com/Hex-Dragon/PCL2)和[PCL2-CE](https://github.com/PCL-Community/PCL2-CE)为蓝本。旨在为各PCL分支版本提供一个标准化的原型样本。该仓库使用 Vue3 搭建，如果你的仓库使用 Webview 作为前端，则可以直接引用该项目。

你可以前往 [PCL.Proto](https://www.amagicpear.sbs/PCL.Proto/) 在线查看本项目。

## 原型

如果你的 PCL 分支版本并非基于 Web 技术，则直接使用本仓库可能并不显得那么方便。那么，你可以前往 [即时设计 - 「PCL.Prototype」](https://js.design/f/QVPQRY?p=zX2rcVk6Cy&mode=design)查看具体的应用原型，并切图导出或参考其生成的样式代码。

如果你想协助修改原型设计文件，欢迎[点击此链接](https://js.design/ti?c=tS-6qs0WDQJ3H4)加入团队。

## 项目配置

目前，项目已成为 Tauri + Vue3 + Vite 结合项目。在首次运行本项目之前，请先运行下面的命令安装前端依赖：

```sh
bun install
```

后端依赖`cargo`会自动安装，无需手动管理。

然后，使用 `bun run tauri icon`自动生成图标。如果不进行此步，在 **Tauri 应用启动时会报错**！接下来就可以开始正式的开发和构建了。

要

### 开发时热重载

```sh
bun run tauri dev
```

## 项目打包求助！

我想把这个项目做成 Vue 组件的 npm 包，但是我不会做，如果有大佬会做组件库的话，欢迎联系我！！！

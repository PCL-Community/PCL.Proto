<script setup lang="ts">
import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import PCard from '@/components/widget/PCard.vue'
import picLongTengMaoYue from '@/assets/pictures/龙腾猫跃头像.jpg'
import picPCLProto from '/PCL.Proto.svg?url'
import tsLogo from '@/assets/icons/ts-logo-256.svg?url'
import PHint from '@/components/widget/PHint.vue'
declare const __APP_VERSION__: string
const version = __APP_VERSION__
</script>

<template lang="pug">
    PCard
        template(#title) 关于
        template(#content)
            CardInfoItem(:icon="picLongTengMaoYue"
                title="龙腾猫跃"
                subtitle="Plain Craft Launcher 的原作者！"
                :roundImg="true"
                :btn="{text: '赞助作者', link:'https://afdian.com/a/LTCat'}"
                )
            CardInfoItem(icon="https://avatars.githubusercontent.com/u/63146698?v=4"
                title="一只会魔法的梨 | AMagicPear"
                subtitle="PCL.Proto 的作者以及 Logo 设计者～"
                :roundImg="true"
                :btn="{text: '去我的主页看看 👀', link:'https://www.amagicpear.sbs/'}"
                )
            CardInfoItem(:icon="picPCLProto"
                title="Plain Craft Launcher: Edition Prototype"
                :subtitle="`当前版本：${version}`"
                :roundImg="true"
                :btn="{text: 'GitHub 仓库', link:'https://github.com/PCL-Community/PCL.Proto.git'}"
                )
    PCard
        template(#title) 特别鸣谢
        template(#content)
            CardInfoItem(icon="https://avatars.githubusercontent.com/u/165488354?s=200&v=4"
                title="PCL Community"
                subtitle="Plain Craft Launcher 社区"
                :btn="{text: 'GitHub 主页', link:'https://github.com/PCL-Community'}"
                )
            CardInfoItem(icon="https://github.com/PCL-Community/PCL.Neo/raw/main/Icon.svg"
                title="PCL.Neo"
                subtitle="感谢 PCL.Neo 带我进入了这个社区！"
                :btn="{text: 'GitHub 仓库', link:'https://github.com/PCL-Community/PCL.Neo'}"
                )
    PCard
        template(#title) 法律信息
        template(#content)
            p 本软件为原版 PCL 的社区衍生版本之一。仓库代码开源，但倘若您需要使用此项目或参考本仓库的源码，需遵守#[a(href="https://github.com/PCL-Community/PCL2-CE/blob/dev/LICENCE") 与原版 PCL 相同的协议]。
            p 本项目主导社区为 PCL-Community，如果你有对此项目的建议或在使用中遇到问题，欢迎提 Issue 和 Pull Request！
 
    PCard(defaultFoldStatus='fold' title="直接使用的项目许可与版权声明")
        PHint(severity="info") 如果我看得更远，那是因为我站在巨人的肩膀上。 —— Sir Isaac Newton
        CardInfoItem(title="Vue.js - 渐进式 JavaScript 框架" subtitle="MIT | https://github.com/vuejs/core" icon="https://vuejs.org/logo.svg")
        CardInfoItem(title="TypeScript - 类型！类型！我要类型！" subtitle="Apache License 2.0 | https://github.com/microsoft/TypeScript" :icon="tsLogo")
        CardInfoItem(title="Pug - 排版特别好看的模板引擎" subtitle="MIT | https://github.com/pugjs/pug" icon="https://www.pugjs.cn/img/logo.png")
        CardInfoItem(title="Vite - 快速、轻量级的开发工具" subtitle="MIT | https://github.com/vitejs/vite" icon="https://vitejs.dev/logo.svg")
        CardInfoItem(title="Vue Router" subtitle="MIT | https://github.com/vuejs/vue-router-next" icon="https://router.vuejs.org/logo.svg")
        CardInfoItem(title="xml-js - XML解析 用于展示PCL原版格式自定义主页" subtitle="MIT | https://www.npmjs.com/package/xml-js" icon="http://nashwaan.github.io/xml-js/images/logo.svg")
        CardInfoItem(title="Pinia - 状态管理" subtitle="MIT |https://pinia.vuejs.org/" icon="https://pinia.vuejs.org/logo.svg")
        CardInfoItem(title="skinview3d - 3D 皮肤查看器" subtitle="MIT | https://github.com/bs-community/skinview3d" icon="https://avatars.githubusercontent.com/u/37259779?s=200&v=4")
        //- CardInfoItem(title="Bun" subtitle="MIT | https://bun.com/" icon="https://bun.com/logo.svg")
        CardInfoItem(title="Rust Programming Language" subtitle="Apache 2.0 & MIT | https://www.rust-lang.org/" icon="https://www.rust-lang.org/logos/rust-logo-blk.svg")
        CardInfoItem(title="Tauri" subtitle="MIT | https://tauri.app/" icon="https://tauri.app/_astro/logo_light.C7Zm2ZoX.svg")
        CardInfoItem(title="EasyTier" subtitle="LGPL-3.0 | https://github.com/EasyTier/EasyTier" icon="https://easytier.cn/easytier.png")
    
    PCard(defaultFoldStatus="fold" title="参考项目实现")
        CardInfoItem(title="PCL-Community/PCL.Core" subtitle="Apache-2.0 | https://github.com/PCL-Community/PCL.Core")
        CardInfoItem(title="PCL-Community/PCL2-CE" subtitle="https://github.com/PCL-Community/PCL2-CE" icon="https://www.pclc.cc/img/pcl-ce/icon.webp")
        CardInfoItem(title="xphost008/MMCLL" subtitle="MIT | https://github.com/xphost008/MMCLL")
        CardInfoItem(title="PCL-Community/PCL.Neo/PCL.Neo.Core" subtitle="MIT | https://github.com/PCL-Community/PCL.Neo")
        CardInfoItem(title="Steve-xmh/scl" subtitle="LGPL-3.0 & SCL License | https://github.com/Steve-xmh/scl" icon="https://steve-xmh.github.io/scl/assets/scl-icon.svg")
</template>

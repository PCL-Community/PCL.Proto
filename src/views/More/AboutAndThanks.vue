<script setup lang="ts">
import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import PCard from '@/components/widget/PCard.vue'
import picLongTengMaoYue from '@/assets/pictures/龙腾猫跃头像.jpg'
import picPCLProto from '/PCL.Proto.svg?url'
import PlainTextInfoItem from '@/components/widget/PlainTextInfoItem.vue'
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
                infoType="about"
                :btn="{text: '赞助作者', link:'https://afdian.com/a/LTCat'}"
                )
            CardInfoItem(icon="https://avatars.githubusercontent.com/u/63146698?v=4"
                title="一只会魔法的梨 | AMagicPear"
                subtitle="PCL.Proto 的作者以及 Logo 设计者～"
                infoType="about"
                :btn="{text: '去我的主页看看 👀', link:'https://www.amagicpear.sbs/'}"
                )
            CardInfoItem(:icon="picPCLProto"
                title="Plain Craft Launcher: Edition Prototype"
                :subtitle="`当前版本：${version}`"
                infoType="about"
                :btn="{text: 'GitHub 仓库', link:'https://github.com/PCL-Community/PCL.Proto.git'}"
                )
    PCard
        template(#title) 特别鸣谢
        template(#content)
            CardInfoItem(icon="https://avatars.githubusercontent.com/u/165488354?s=200&v=4"
                title="PCL Community"
                subtitle="Plain Craft Launcher 社区"
                infoType="about"
                :btn="{text: 'GitHub 主页', link:'https://github.com/PCL-Community'}"
                )
            CardInfoItem(icon="https://github.com/PCL-Community/PCL.Neo/raw/main/Icon.svg"
                title="PCL.Neo"
                subtitle="本项目使用 PCL.Neo.Core 作为后端启动核心"
                infoType="about"
                :btn="{text: 'GitHub 仓库', link:'https://github.com/PCL-Community/PCL.Neo'}"
                )
    PCard
        template(#title) 法律信息
        template(#content)
            p 本软件为原版 PCL 的社区衍生版本之一。仓库代码开源，但倘若您需要使用此项目或参考本仓库的源码，需遵守#[a(href="https://github.com/PCL-Community/PCL2-CE/blob/dev/LICENCE") 与原版 PCL 相同的协议]。
            p 本项目主导社区为 PCL-Community，如果你有对此项目的建议或在使用中遇到问题，欢迎提 Issue 和 Pull Request！
 
    PCard(defaultFoldStatus='fold')
        template(#title) 许可与版权声明
        template(#content)
            PlainTextInfoItem
                template(#title) Vue.js - 渐进式 JavaScript 框架
                template(#content) MIT | https://github.com/vuejs/core
            PlainTextInfoItem
                template(#title) TypeScript - 类型！类型！我要类型！
                template(#content) Apache License 2.0 | https://github.com/microsoft/TypeScript
            PlainTextInfoItem
                template(#title) Pug - 排版特别好看的模板引擎
                template(#content) MIT | https://github.com/pugjs/pug
            PlainTextInfoItem
                template(#title) Vite - 快速、轻量级的开发工具
                template(#content) MIT | https://github.com/vitejs/vite
            PlainTextInfoItem
                template(#title) Vue Router
                template(#content) MIT | https://github.com/vuejs/vue-router-next
            PlainTextInfoItem
                template(#title) PCL.Neo.Core
                template(#content) MIT | https://github.com/PCL-Community/PCL.Neo/
</template>

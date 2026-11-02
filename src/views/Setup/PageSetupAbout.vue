<script setup lang="ts">
import { pclText } from '@/api/pclLocale'
import PListItem from '@/components/widget/PListItem.vue'
import PCard from '@/components/widget/PCard.vue'
import picLongTengMaoYue from '@/assets/pictures/龙腾猫跃头像.jpg'
import picPCLProto from '/PCL.Proto.svg?url'
import tsLogo from '@/assets/icons/ts-logo-256.svg?url'
import terracottaLogo from '@/assets/icons/terracotta-icon.png'
import PHint from '@/components/widget/PHint.vue'
declare const __APP_VERSION__: string
const version = __APP_VERSION__
</script>

<template lang="pug">
section.pcl-page.proto-about
    PCard
        template(#title) {{ pclText('Setup.About.Title') }}
        template(#content)
            PListItem(:icon="picLongTengMaoYue"
                title="龙腾猫跃"
                :subtitle="pclText('Setup.About.OriginalAuthor.Info')"
                :roundImg="true"
                :btn="{text: pclText('Proto.About.Sponsor'), link:'https://afdian.com/a/LTCat'}"
                )
            PListItem(icon="https://avatars.githubusercontent.com/u/63146698?v=4"
                title="一只会魔法的梨 | AMagicPear"
                :subtitle="pclText('Proto.About.ProtoAuthor')"
                :roundImg="true"
                :btn="{text: pclText('Proto.About.Website'), link:'https://amagicpear.top/'}"
                )
            PListItem(:icon="picPCLProto"
                title="Plain Craft Launcher: Edition Prototype"
                :subtitle="pclText('Proto.About.Version', [version])"
                :roundImg="true"
                :btn="{text: pclText('Proto.About.Repository'), link:'https://github.com/PCL-Community/PCL.Proto.git'}"
                )
    PCard
        template(#title) {{ pclText('Setup.About.SpecialThanks.Title') }}
        template(#content)
            PListItem(icon="https://avatars.githubusercontent.com/u/165488354?s=200&v=4"
                title="PCL Community"
                :subtitle="pclText('Proto.About.Community')"
                :btn="{text: pclText('Setup.About.GitHubHome'), link:'https://github.com/PCL-Community'}"
                )
            PListItem(icon="https://github.com/PCL-Community/PCL.Neo/raw/main/Icon.svg"
                title="PCL.Neo"
                :subtitle="pclText('Proto.About.NeoThanks')"
                :btn="{text: pclText('Proto.About.Repository'), link:'https://github.com/PCL-Community/PCL.Neo'}"
                )
    PCard
        template(#title) {{ pclText('Proto.About.Legal') }}
        template(#content)
            p
                | {{ pclText('Proto.About.LegalIntro') }}
                | #[a(href="https://github.com/PCL-Community/PCL2-CE/blob/dev/LICENCE") {{ pclText('Proto.About.LegalLink') }}]
            p {{ pclText('Proto.About.Contribute') }}

    PCard(defaultFoldStatus='fold' :title="pclText('Proto.About.Licenses')")
        PHint(severity="info") {{ pclText('Proto.About.Newton') }}
        PListItem(:title="pclText('Proto.About.Vue')" subtitle="MIT | https://github.com/vuejs/core" icon="https://vuejs.org/logo.svg")
        PListItem(:title="pclText('Proto.About.Lucide')" subtitle="ISC | https://lucide.dev/")
        PListItem(:title="pclText('Proto.About.TypeScript')" subtitle="Apache License 2.0 | https://github.com/microsoft/TypeScript" :icon="tsLogo")
        PListItem(:title="pclText('Proto.About.Pug')" subtitle="MIT | https://github.com/pugjs/pug" icon="https://www.pugjs.cn/img/logo.png")
        PListItem(:title="pclText('Proto.About.Vite')" subtitle="MIT | https://github.com/vitejs/vite" icon="https://vitejs.dev/logo.svg")
        PListItem(title="Vue Router" subtitle="MIT | https://github.com/vuejs/vue-router-next" icon="https://router.vuejs.org/logo.svg")
        PListItem(:title="pclText('Proto.About.Xml')" subtitle="MIT | https://www.npmjs.com/package/xml-js" icon="http://nashwaan.github.io/xml-js/images/logo.svg")
        PListItem(:title="pclText('Proto.About.Pinia')" subtitle="MIT |https://pinia.vuejs.org/" icon="https://pinia.vuejs.org/logo.svg")
        PListItem(:title="pclText('Proto.About.SkinView')" subtitle="MIT | https://github.com/bs-community/skinview3d" icon="https://avatars.githubusercontent.com/u/37259779?s=200&v=4")
        //- PListItem(title="Bun" subtitle="MIT | https://bun.com/" icon="https://bun.com/logo.svg")
        PListItem(title="Rust Programming Language" subtitle="Apache 2.0 & MIT | https://www.rust-lang.org/" icon="https://www.rust-lang.org/logos/rust-logo-blk.svg")
        PListItem(title="Tauri" subtitle="MIT | https://tauri.app/" icon="https://tauri.app/_astro/logo_light.C7Zm2ZoX.svg")
        //- PListItem(title="EasyTier" subtitle="LGPL-3.0 | https://github.com/EasyTier/EasyTier" icon="https://easytier.cn/easytier.png")
        PListItem(:icon="terracottaLogo" title="burningtnt/Terracotta" subtitle="AGPL-3.0 license | https://github.com/burningtnt/Terracotta.git")

    PCard(defaultFoldStatus="fold" :title="pclText('Proto.About.References')")
        PListItem(title="PCL-Community/PCL.Core" subtitle="Apache-2.0 | https://github.com/PCL-Community/PCL.Core")
        PListItem(title="PCL-Community/PCL2-CE" subtitle="https://github.com/PCL-Community/PCL2-CE" icon="https://www.pclc.cc/img/pcl-ce/icon.webp")
        PListItem(title="xphost008/MMCLL" subtitle="MIT | https://github.com/xphost008/MMCLL")
        PListItem(title="PCL-Community/PCL.Neo/PCL.Neo.Core" subtitle="MIT | https://github.com/PCL-Community/PCL.Neo")
        PListItem(title="Steve-xmh/scl" subtitle="LGPL-3.0 & SCL License | https://github.com/Steve-xmh/scl" icon="https://steve-xmh.github.io/scl/assets/scl-icon.svg")

</template>

<style scoped>
.proto-about {
  padding: 25px;
  display: flex;
  flex-direction: column;
  gap: 15px;
}
.proto-about :deep(.mycard-content) {
  padding: 0 20px 20px;
}
</style>

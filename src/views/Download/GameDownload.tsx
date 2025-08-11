import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import PCard from '@/components/widget/PCard.vue'
import { showIconPath, type showIconType } from '@/util/gameInfo'
import { getMinecraftVersions } from '@/api/gameVersions'
import { defineComponent, onMounted } from 'vue'
import PLoading from '@/components/widget/PLoading.vue'
let { cacheVersionData, versionDataRef } = getMinecraftVersions()

function clickOnVersion(version: string) {
  console.log('点击了版本', version)
}

const renderVersionSection = (
  title: string,
  dataKey: 'release' | 'snapshot' | 'old',
  icon: showIconType,
) => (
  <PCard defaultFoldStatus="fold">
    {{
      title: () => `${title} (${versionDataRef.value?.[dataKey]?.length || 0})`,
      content: () =>
        versionDataRef.value?.[dataKey]?.map((item) => (
          <CardInfoItem
            key={item.id}
            icon={showIconPath[icon]}
            title={item.id}
            subtitle={`发布于 ${item.releaseTime}`}
            isGameInfo
            click={() => clickOnVersion(item.id)}
          />
        )),
    }}
  </PCard>
)

export default defineComponent({
  setup() {
    onMounted(() => {
      if (!cacheVersionData) {
        console.warn('缓存不存在')
      }
    })
    return () =>
      versionDataRef.value ? (
        <>
          <PCard title="最新版本">
            <CardInfoItem
              icon={showIconPath.grass}
              title={versionDataRef.value.latest.release.id}
              subtitle={`最新正式版，发布于 ${versionDataRef.value.latest.release.releaseTime}`}
              is-game-info
              click={() => clickOnVersion(versionDataRef.value?.latest.release.id as string)}
            />
            <CardInfoItem
              icon={showIconPath.command}
              title={versionDataRef.value.latest.snapshot.id}
              subtitle={`最新预览版，发布于 ${versionDataRef.value.latest.snapshot.releaseTime}`}
              is-game-info
              click={() => clickOnVersion(versionDataRef.value?.latest.snapshot.id as string)}
            />
          </PCard>

          {renderVersionSection('正式版', 'release', 'grass')}

          {renderVersionSection('预览版', 'snapshot', 'command')}

          {renderVersionSection('远古版', 'old', 'stone')}

          <PCard defaultFoldStatus="unfoldable" title="愚人节版 (尚未分类)" />
        </>
      ) : (
        <div class="loading-page">
          <PLoading state="loading" />
        </div>
      )
  },
})

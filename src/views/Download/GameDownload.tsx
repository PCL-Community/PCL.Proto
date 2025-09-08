import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import PCard from '@/components/widget/PCard.vue'
import { showIconPath, type showIconType } from '@/util/gameInfo'
import { getMinecraftVersions } from '@/api/gameVersions'
import { defineComponent, onMounted } from 'vue'
import PLoading from '@/components/widget/PLoading.vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
let { cacheVersionData, versionDataRef } = getMinecraftVersions()

export default defineComponent({
  setup() {
    const router = useRouter()
    const { t } = useI18n()
    onMounted(() => {
      if (!cacheVersionData) {
        console.warn('缓存不存在')
      }
    })

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
                subtitle={`${t('download.published_at')} ${item.releaseTime}`}
                isGameInfo
                click={() => clickOnVersion(item.id)}
              />
            )),
        }}
      </PCard>
    )
    function clickOnVersion(version: string) {
      console.log('点击了版本', version, router)
      router.push({
        path: '/downloading',
        // query: {
        //   version,
        // },
      })
    }
    return () =>
      versionDataRef.value ? (
        <>
          <PCard title={t('download.latest')}>
            <CardInfoItem
              icon={showIconPath.grass}
              title={versionDataRef.value.latest.release.id}
              subtitle={`${t('download.latest_stable')}, ${t('download.published_at')} ${versionDataRef.value.latest.release.releaseTime}`}
              is-game-info
              click={() => clickOnVersion(versionDataRef.value?.latest.release.id as string)}
            />
            <CardInfoItem
              icon={showIconPath.command}
              title={versionDataRef.value.latest.snapshot.id}
              subtitle={`${t('download.latest_snapshot')}, ${t('download.published_at')} ${versionDataRef.value.latest.snapshot.releaseTime}`}
              is-game-info
              click={() => clickOnVersion(versionDataRef.value?.latest.snapshot.id as string)}
            />
          </PCard>

          {renderVersionSection(t('download.stable'), 'release', 'grass')}

          {renderVersionSection(t('download.snapshot'), 'snapshot', 'command')}

          {renderVersionSection(t('download.old'), 'old', 'stone')}

          <PCard defaultFoldStatus="unfoldable" title={t('download.april')} />
        </>
      ) : (
        <div class="loading-page">
          <PLoading state="loading" />
        </div>
      )
  },
})

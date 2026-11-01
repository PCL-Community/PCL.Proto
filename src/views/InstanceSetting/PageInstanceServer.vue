<script setup lang="ts">
import { showUnavailable } from '@/util/showUnavailable'
import { pclText } from '@/api/pclLocale'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import ServerCard from '@/components/widget/ServerCard.vue'
withDefaults(
  defineProps<{
    servers?: {
      name: string
      address: string
      icon?: string
      motd?: string
      players?: string
      latency?: number
    }[]
  }>(),
  { servers: () => [] },
)

const text = pclText
</script>
<template>
  <div class="pcl-page">
    <div>
      <div data-control="PanBack">
        <div style="display: grid">
          <PCard
            v-if="!servers.length"
            data-control="PanNoServer"
            style="
              margin: 40px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
              justify-self: center;
              align-self: center;
            "
            hide-title
          >
            <div
              style="
                margin: 17px 20px;
                display: grid;
                grid-template-columns: minmax(0, 1fr) auto auto minmax(0, 1fr);
                grid-template-rows: auto auto auto auto;
              "
            >
              <span
                style="
                  margin: 0px 0px 9px 0px;
                  font-size: 19px;
                  grid-row: 1 / span 1;
                  grid-column: 1 / span 4;
                  justify-self: center;
                  color: var(--color-tint);
                "
                class="pcl-text"
                >{{ text('Instance.Server.Empty.Title') }}</span
              >
              <div
                style="
                  height: 2px;
                  grid-row: 2 / span 1;
                  grid-column: 1 / span 4;
                  justify-self: stretch;
                "
                class="pcl-divider"
              />
              <span
                style="margin: 15px 10px 5px 10px; grid-row: 3 / span 1; grid-column: 1 / span 4"
                class="pcl-text"
                >{{ text('Instance.Server.Empty.Message') }}</span
              >
              <PButton
                style="
                  height: 35px;
                  min-width: 140px;
                  margin: 10px 10px 0px 10px;
                  padding: 0px 13px;
                  grid-row: 4 / span 1;
                  grid-column: 2 / span 1;
                  justify-self: center;
                "
                @click="showUnavailable"
                type="tint"
                >{{ text('Instance.Server.RefreshInfo') }}</PButton
              >
              <PButton
                data-control="BtnAddServerTop"
                style="
                  height: 35px;
                  min-width: 140px;
                  margin: 10px 10px 0px 10px;
                  padding: 0px 13px;
                  grid-row: 4 / span 1;
                  grid-column: 3 / span 1;
                  justify-self: center;
                "
                @click="showUnavailable"
                >{{ text('Instance.Server.AddServer') }}</PButton
              >
            </div>
          </PCard>
          <div
            v-if="servers.length"
            style="
              margin: 25px 10px 10px 10px;
              grid-row: 1 / span 1;
              grid-column: 1 / span 1;
              display: flex;
              flex-direction: column;
            "
          >
            <PCard :title="text('Instance.Server.QuickActions')"
              ><div class="server-actions">
                <PButton type="tint" @click="showUnavailable">{{
                  text('Instance.Server.RefreshAll')
                }}</PButton
                ><PButton @click="showUnavailable">{{ text('Instance.Server.AddServer') }}</PButton>
              </div></PCard
            >
            <ServerCard
              v-for="server in servers"
              :key="server.address"
              v-bind="server"
              @connect="showUnavailable()"
              @refresh="showUnavailable()"
              @edit="showUnavailable()"
              @remove="showUnavailable()"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.server-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 15px;
  padding: 0 25px 15px;
}
.server-actions :deep(button) {
  min-width: 140px;
  min-height: 35px;
}
</style>

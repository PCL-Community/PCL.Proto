<script setup lang="ts">
import { invoke, isTauri } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { pclText } from "@/api/pclLocale";
import sideTip from "@/composables/sideTip";
import { openExternal } from "@/util/externalLink";
import PSvgIcon from "./PSvgIcon.vue";
import { ref } from "vue";
import { downloadForgelike } from "@/api/forgelikeDownload";
import { isTaskCancelled } from "@/stores/task";

const props = defineProps<{ loader: string; game: string; version: string }>();
const saving = ref(false);
const family = () => props.loader.toLowerCase();
async function saveFile() {
  if (saving.value) return;
  if (!isTauri())
    return sideTip.show(pclText("Proto.Desktop.Download"), "warn");
  const name = `fabric-loader-${props.version}.jar`;
  saving.value = true;
  try {
    if (["forge", "neoforge"].includes(family())) {
      const destination = await downloadForgelike(
        props.loader,
        props.game,
        props.version,
      );
      if (destination)
        sideTip.show(pclText("Proto.Download.Saved", [destination]), "success");
      return;
    }
    const destination = await save({
      defaultPath: name,
      filters: [{ name: "JAR", extensions: ["jar"] }],
    });
    if (!destination) return;
    sideTip.show(
      pclText("Download.Comp.QuickDownload.Hint.DownloadStarted", [
        props.version,
      ]),
    );
    await invoke("save_loader_file", {
      loader: family(),
      game: props.game,
      version: props.version,
      destination,
    });
    sideTip.show(pclText("Proto.Download.Saved", [destination]), "success");
  } catch (error) {
    if (!isTaskCancelled(error))
      sideTip.show(pclText("Proto.Error.Download", [String(error)]), "warn");
  } finally {
    saving.value = false;
  }
}
async function changelog() {
  try {
    if (isTauri())
      await invoke("open_loader_changelog", {
        loader: family(),
        game: props.game,
        version: props.version,
      });
    else {
      const id = `${encodeURIComponent(props.game)}-${encodeURIComponent(props.version)}`;
      void openExternal(
        family() === "forge"
          ? `https://files.minecraftforge.net/maven/net/minecraftforge/forge/${id}/forge-${id}-changelog.txt`
          : "https://fabricmc.net/blog",
      );
    }
  } catch (error) {
    sideTip.show(
      `${pclText("Instance.Resource.Item.OpenChangelogFailed")}: ${String(error)}`,
      "warn",
    );
  }
}
</script>
<template>
  <button
    class="pcl-icon-button"
    type="button"
    :disabled="saving"
    :title="pclText('Download.Version.SaveAs')"
    :aria-label="pclText('Download.Version.SaveAs')"
    @click="saveFile"
  >
    <PSvgIcon name="lucide/save" :size="15" />
  </button>
  <button
    v-if="family() !== 'neoforge'"
    class="pcl-icon-button"
    type="button"
    :title="pclText('Download.Version.Changelog')"
    :aria-label="pclText('Download.Version.Changelog')"
    @click="changelog"
  >
    <PSvgIcon name="lucide/info" :size="15" />
  </button>
</template>

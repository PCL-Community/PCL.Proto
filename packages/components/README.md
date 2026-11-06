# PCL.Proto Components

Vue 3 components from PCL.Proto. See the [repository](https://github.com/AMagicPear/PCL.Proto) for development and the full license. The package must be published to npm before the following registry install command can be used; for a local build, run `npm pack ./packages/components` from the repository root and install the resulting tarball instead.

```sh
npm install @amagicpear/pcl-proto-components vue
```

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { PButton, PTextBox } from '@amagicpear/pcl-proto-components'
import '@amagicpear/pcl-proto-components/style.css'

const name = ref('Player')
</script>

<template>
  <div class="theme-blue">
    <PTextBox v-model="name" aria-label="Player name" />
    <PButton type="tint">Hello {{ name }}</PButton>
  </div>
</template>
```

Exports: `PButton`, `PCard`, `PCheckBox`, `PComboBox`, `PErrorRetry`, `PHint`, `PIconTextButton`, `PListActions`, `PListItem`, `PLoading`, `PPath`, `PPrimaryAction`, `PSearchBox`, `PSelectionIndicator`, `PSelectionToolbar`, `PSvgIcon`, `PTextBox`, `PVirtualList`, and `SetupItem`. `PVirtualList` expects a scrollable ancestor with the `subview` class and fixed-height rows. Default component text is Chinese; call `await setPclLocale('en-US')` to change it. The stylesheet includes the application's global reset and theme variables. Set the `theme-blue` class on a containing element; palette variables can be overridden with `--ce-color-1` through `--ce-color-8`.

The package is subject to the repository's `LICENSE`; review its usage terms before distribution.

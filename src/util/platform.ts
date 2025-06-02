import { platform } from '@tauri-apps/plugin-os';
import { ref } from 'vue';
const currentPlatform = ref(platform())

export default currentPlatform
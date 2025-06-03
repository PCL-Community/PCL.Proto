import { computed, ref } from "vue";

export const apiUrlInput = ref('http://localhost:5064/api/');

export const apiUrl = computed(() => {
    let url = apiUrlInput.value;
    if (!url.endsWith('/')) {
        url += '/';
    }
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
        url = 'http://' + url;
    }
    return url;
});
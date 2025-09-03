import { computed, ref } from "vue";

export const localApiInput = ref('http://localhost:5064/api/');

export const localApiEndpoint = computed(() => {
    let url = localApiInput.value;
    if (!url.endsWith('/')) {
        url += '/';
    }
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
        url = 'http://' + url;
    }
    return url;
});
import { reactive } from "vue";



export const setupOptions = reactive({
    lauch: {
        defaultVersionIsolaion: 'disabled',
        gameWindowTitle: 'default',
        customInfo: 'PCL.Proto',
    },
    gameMemory: {
        mode: 'auto',
    }
})
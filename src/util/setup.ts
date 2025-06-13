import { reactive, watch } from "vue";

export interface ISetupOption {
    key: string;
    text: string;
}

export type SetupItemType = 'select' | 'input'

export const setupOptions = {
    launch: {
        defaultVersionIsolation: {
            type: 'select' as SetupItemType,
            label: '默认版本隔离',
            options: [
                { key: 'disabled', text: '关闭' },
                { key: 'isolate-moddable', text: '隔离可安装 Mod 的版本' },
                { key: 'isolate-nonformal', text: '隔离非正式版' },
                { key: 'isolate-moddable-and-nonformal', text: '隔离可安装 Mod 的版本与非正式版本' },
                { key: 'isolate-all', text: '隔离所有版本' }
            ] as ISetupOption[],
        },
        gameWindowTitle: {
            type: 'input' as SetupItemType,
            options: [{ key: 'placeholder', text: '默认' }],
            label: '游戏窗口标题',
        },
        customInfo: {
            type: 'input' as SetupItemType,
            options: [{ key: 'placeholder', text: '默认' }],
            label: '自定义信息',
        },
        launcherVisibility: {
            type: 'select' as SetupItemType,
            label: '启动器可见性',
            options: [
                { key: 'close-immediately', text: '游戏启动后立即关闭' },
            ] as ISetupOption[],
        }
    }
};

// 导出响应式设置对象，需要设置默认值要在这里设置
export const setup = reactive({
    launch: {
        defaultVersionIsolation: 'isolate-all',
        gameWindowTitle: '',
        customInfo: 'PCL.Proto',
        launcherVisibility: 'close-immediately',
    },
    gameMemory: {
        mode: 'auto' as 'auto' | 'custom',
        percent: 0,
        optimizeBeforeStart: true,
    }
});

watch(setup, () => {
    // localStorage.setItem('setup.launch', JSON.stringify(setup.launch));
    console.log('Setup changed:', setup);
}, { deep: true });
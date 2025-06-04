import { reactive, watch } from "vue";

export interface ISetupOption {
    text: string;
    value: string;
}

interface ISetupField<T extends string = string> {
    text: string;
    options: { value: T; text: string }[];
    default: T;
}

// 让 ILaunchSetup 泛型化以便与 setupOptions 保持一致
export type LaunchSetupTypes = {
    launch: {
        defaultVersionIsolation: 'disabled' | 'isolate-moddable' | 'isolate-nonformal' | 'isolate-moddable-and-nonformal' | 'isolate-all';
        gameWindowTitle: 'default' | 'custom';
    };
    gameMemory: {
        mode: 'default' | 'custom';
    };
};

// 用映射类型让 setupOptions 与 ILaunchSetup 保持一致
export const setupOptions: { [K in keyof LaunchSetupTypes]: {
    [F in keyof LaunchSetupTypes[K]]: ISetupField<Extract<LaunchSetupTypes[K][F], string>>; } } = {
    launch: {
        defaultVersionIsolation: {
            text: '默认版本隔离',
            options: [
                { value: 'disabled', text: '关闭' },
                { value: 'isolate-moddable', text: '隔离可安装 Mod 的版本' },
                { value: 'isolate-nonformal', text: '隔离非正式版' },
                { value: 'isolate-moddable-and-nonformal', text: '隔离可安装 Mod 的版本与非正式版本' },
                { value: 'isolate-all', text: '隔离所有版本' }
            ],
            default: 'isolate-all'
        },
        gameWindowTitle: {
            text: '游戏窗口标题',
            options: [
                { value: 'default', text: '默认' },
                { value: 'custom', text: '自定义' }
            ],
            default: 'default'
        }
    },
    gameMemory: {
        mode: {
            text: '内存分配模式',
            options: [
                { value: 'default', text: '默认' },
                { value: 'custom', text: '自定义' }
            ],
            default: 'default'
        }
    }
};

// 初始化设置
function initializeSetup(): LaunchSetupTypes {
    return Object.entries(setupOptions).reduce((acc, [category, fields]) => {
        acc[category] = Object.entries(fields).reduce((categoryAcc: Record<string, any>, [field, config]) => {
            categoryAcc[field] = config.default as any;
            return categoryAcc;
        }, {});
        return acc;
    }, {} as any) as LaunchSetupTypes;
}

// 导出响应式设置对象
export const setup = reactive<LaunchSetupTypes>(initializeSetup());

// 添加持久化相关的工具函数
export const setupUtils = {
    saveToStorage() {
        localStorage.setItem('pcl-setup', JSON.stringify(setup));
    },

    loadFromStorage(): Partial<LaunchSetupTypes> | null {
        const stored = localStorage.getItem('pcl-setup');
        return stored ? JSON.parse(stored) : null;
    },

    resetToDefault() {
        const defaultSetup = initializeSetup();
        Object.assign(setup, defaultSetup);
    }
};

watch(setup, () => {
    // localStorage.setItem('setup.launch', JSON.stringify(setup.launch));
    console.log('Setup changed:', setup);
}, { deep: true });
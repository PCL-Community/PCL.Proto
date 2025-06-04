import { reactive } from "vue";

export interface SetupOption {
    text: string;
    value: string;
}

interface SetupField {
    text: string;
    options: SetupOption[];
    default: string;
}

interface SetupCategory {
    [key: string]: SetupField;
}

export const setupOptions: Record<string, SetupCategory> = {
    launch: {
        defaultVersionIsolation: {
            text: '默认版本隔离',
            options: [
                { value: 'disabled', text: '关闭' },
                { value: 'isolate-moddable', text: '隔离可安装 Mod 的版本与非正式版' }
            ],
            default: 'disabled'
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

export const setup = reactive(
    Object.entries(setupOptions).reduce((acc, [category, fields]) => {
        acc[category] = Object.entries(fields).reduce((categoryAcc, [field, config]) => {
            categoryAcc[field] = config.default;
            return categoryAcc;
        }, {} as Record<string, string>);
        return acc;
    }, {} as Record<string, Record<string, string>>)
);
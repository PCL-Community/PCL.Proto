import { apiUrl } from "./apiUrl";

export const archMap: Record<number, string> = {
    0: 'x86',
    1: 'x64',
    2: 'arm64',
    3: 'FatFile',
    4: 'Unknown'
};

export interface IJavaRuntimeInfo {
    directoryPath: string,
    isUserImport: boolean,
    version: string,
    slugVersion: number,
    is64Bit: boolean,
    architecture: number,
    compability: number,
    isJre: boolean,
    javaExe: string,
    javaWExe: string,
    implementor: string,
}
export async function getJavaList(): Promise<IJavaRuntimeInfo[]> {
    const res = await fetch(new URL('javalist', apiUrl.value));
    const data = await res.json();
    return data;
}
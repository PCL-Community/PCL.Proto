// import { localApiEndpoint } from "./localApi";

import { invoke } from "@tauri-apps/api/core";

export interface IJavaRuntimeInfo {
    directory_path: string,
    is_user_imported: boolean,
    version: string,
    slug_version: number,
    is_64_bit: boolean,
    architecture: 'X86' | 'X64' | 'Arm64' | 'FatFile' | 'Unknown',
    compability: number,
    is_jdk: boolean,
    java_exe: string,
    implementor: string,
}

async function getJavaList(): Promise<IJavaRuntimeInfo[]> {
    const javaList = await invoke<IJavaRuntimeInfo[]>('get_java_list')
    return javaList;
    // const res = await fetch(new URL('java/list', localApiEndpoint.value));
    // const data = await res.json();
    // return data;
}

async function refreshJavaList() {
    return { success: true }
    // const res = await fetch(new URL('java/refresh', localApiEndpoint.value), { method: 'POST' })
    // const data = await res.json();
    // return data
}

export {
    getJavaList,
    refreshJavaList
}
// 示例 api/doSomething.ts
export async function doSomething(msg: string) {
    const res = await fetch('http://localhost:5064/api/do-something', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ module: 'PerryTest', message: msg }),
    })
    const data = await res.json()
    return data
}

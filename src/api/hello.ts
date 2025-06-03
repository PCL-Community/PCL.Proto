// 示例：src/api/hello.ts
export async function getHello() {
    const res = await fetch('http://localhost:5281//weatherforecast');
    const data = await res.json();
    return data;
}

// 示例 api/doSomething.ts
export async function doSomething(msg: string) {
    const res = await fetch('http://localhost:5281/api/do-something', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ message: msg }),
    })
    const data = await res.json()
    return data
}


const result = await doSomething('perry')
const response = JSON.stringify(result)
console.log(response)


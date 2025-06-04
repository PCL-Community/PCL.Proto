<template lang="pug">
    select
        option(value='a') 选项 A
        option(value='b') 选项 B
        option(value='c') 选项 C
</template>

<style lang="css" scoped>
select {
    background-color: transparent;
    border-radius: 4px;
    font-size: 12px;
    padding: 0.3rem 0.8rem;
    border: 1px solid var(--color-tint-lightist);
    transition: background-color 0.3s ease;
    color: var(--color-text-black);
}

select:hover {
    background-color: var(--color-tint-lighter);
}
</style>
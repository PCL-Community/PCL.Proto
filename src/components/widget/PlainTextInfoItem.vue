<template lang="pug">
    .java-info
        p: slot(name="title")
        p.java-info-c: slot(name="content")
</template>

<style lang="css" scoped>
.java-info {
    padding: 4px 6px;
    border-radius: 4px;
    transition: background-color 0.4s ease;
}

.java-info:hover {
    background-color: var(--color-tint-lighter);
}

.java-info-c {
    color: var(--color-text-grey);
}
</style>
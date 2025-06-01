<script lang="ts" setup>
import MyButton from '@/components/widget/MyButton.vue';
import MyCard from '@/components/widget/MyCard.vue';
</script>

<template lang="pug">
    MyCard(:hide-title="false")
        template(#title) 标题
        template(#content)
            p(v-for="_ in 4") www

    MyButton(type="tint") 我的按钮
    MyButton(type="warn") 你的按钮
    MyButton(v-for="i in 12", type="default") 你的按钮
</template>

<script setup>
import { ref } from "vue";

import Tabs from './Tabs.vue'

const greetMsg = ref("");
const name = ref("");
</script>

<template>
  <div @click.right.prevent id="wrapper">
    <Tabs />
  </div>
</template>

<style scoped lang="scss">
@use "styles/colors";

#wrapper {
  height: 100vh;
  user-select: none;
}

</style>
<style src="./styles/main.scss" lang="scss"/>
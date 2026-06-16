<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

import Tabs from './Tabs.vue'

const greetMsg = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsg.value = await invoke("greet", { name: name.value });
}
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
<script setup>
  import {ref} from "vue";

  import {invoke} from '@tauri-apps/api/core';

  let addr = ref("")

  async function request() {
    try {
      await invoke('fetch', {address: addr.value})
    } catch (err) {
      console.log(err);
    }
  }
</script>

<template>
  Inputs Page!<br/>
  <!--TODO: This is a temporary test, remove it when adding final UI-->
  <input type="text" v-model="addr" placeholder="type a command..."><button @click="request">Request</button>
</template>

<style scoped lang="scss">

</style>
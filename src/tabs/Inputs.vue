<script setup>
  import {ref} from "vue";
  import {invoke} from '@tauri-apps/api/core';
  import PatchGrid from "../grids/PatchGrid.vue";

  let inTabs = [
    {name: "Main", len: 32},
    {name: "Aux", len: 6},
  ];

  let inSections = [
    {name: "Local", len: 32},
    {name: "AES50-A", len: 48},
    {name: "AES50-B", len: 48},
    {name: "Card", len: 32},
    {name: "Aux", len: 6},
    {name: "Talkback", len: 2},
  ];

</script>

<template>
  <PatchGrid :tabs="inTabs" :sections="inSections"></PatchGrid>
</template>

<style scoped lang="scss">

</style>
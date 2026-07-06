<script setup>
  import {ref} from "vue";

  import Setup from './tabs/Setup.vue'
  import Inputs from './tabs/Inputs.vue'
  import Outputs from './tabs/Outputs.vue'
  import StatusIndicator from "./components/StatusIndicator.vue";

  let tabs = ref(["Inputs", "Outputs", "Setup"]);
  let activeTabIdx = ref(0);

  function switchTab(index) {
    activeTabIdx.value = index;
  }
</script>

<template>
  <div class="tab-bar">
    <div
        v-for="(title, index) in tabs"
        @click="switchTab(index)"
        :class="{active: (activeTabIdx === index), tab: true}">
      {{title}}
    </div>
    <div class="status">
      <StatusIndicator />
    </div>
  </div>
  <div class="tab-content">
    <Inputs v-if="tabs[activeTabIdx] === 'Inputs'" />
    <Outputs v-if="tabs[activeTabIdx] === 'Outputs'" />
    <Setup v-if="tabs[activeTabIdx] === 'Setup'" />
  </div>
</template>

<style scoped lang="scss">
@use "./styles/colors";

div.tab-bar {
  display: flex;
  flex-direction: row;
  width: 100vw;
  padding: 8px 12px 0 12px;

  div.tab {
    background-color: colors.$tab-bg;
    margin-right: 8px;
    margin-bottom: 4px;
    padding: 6px 20px 6px 20px;
    border-radius: 0.7vh;
    box-shadow: 1px 1px 3px colors.$shadow;
  }

  div.tab:hover {
    transition: background-color 0.2s;
    background-color: colors.$tab-bg-hover;
    cursor: pointer;
  }

  div.tab.active {
    cursor: default;
    background-color: colors.$tab-bg-active;
    margin-bottom: 0;
    padding: 6px 20px 10px 20px;
    border-radius: 0.7vh 0.7vh 0 0;
    box-shadow: none;
  }

  div.tab.active:hover {
    background-color: colors.$tab-bg-active;
  }

  div.tab:active {
    transition: none;
    background-color: colors.$tab-bg-active;
  }

  div.status {
    margin-left: auto;
    align-content: center;
  }
}

div.tab-content {
  height: 90%;
  width: 100vw;
  background-color: colors.$background-light;
  padding: 20px;
}
</style>
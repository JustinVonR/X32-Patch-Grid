<script setup>
import {onMounted, ref} from "vue";

  const props = defineProps(['tabs', 'sections']);
  let activeTabIdx = ref(0);

  let tabStartIds = ref([1]);
  let sectionStartIds = ref([1]);

  onMounted(() => {
    for (let i = 1; i < props.tabs.length; i++) {
      tabStartIds.value.push(tabStartIds.value[i - 1] + props.tabs[i - 1].len);
    }

    for (let i = 1; i < props.sections.length; i++) {
      sectionStartIds.value.push(sectionStartIds.value[i - 1] + props.sections[i - 1].len);
    }
    console.log(sectionStartIds.value);
  })

  function switchActive(idx) {
    activeTabIdx.value = idx;
  }
</script>

<template>
  <div class="grid-wrapper">
    <div class="grid-tabs">
      <div
          v-for="(tab, idx) in props.tabs"
          :class="{
            'grid-tab': true,
            active: (idx === activeTabIdx)}"
          @click="switchActive(idx)"
      >{{ tab.name }} 1-{{ tab.len }}</div>
    </div>
    <div class="grid-content">
      <div
        v-for="(section, idx) in props.sections"
      >
        <div class="section-header">{{ section.name }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
  @use "../styles/colors";

  div.grid-tabs {
    display: flex;
    flex-direction: row-reverse;
    padding-bottom: 40px;
    writing-mode: vertical-lr;
    transform: rotate(180deg);
  }

  div.grid-tab {
    background-color: colors.$background-mid;
    border-radius: 4px;
    margin-top: 12px;
    margin-left: 12px;
    padding: 12px 3px 12px 3px;
    transition: background-color 0.2s;
    border: 1px solid black;
    text-wrap: nowrap;

    &:hover {
      cursor: pointer;
      background-color: colors.$tab-bg-hover;
    }

    &.active {
      background-color: colors.$primary;

      &:hover {
        background-color: colors.$primary-hover;
      }
    }
  }



  div.grid-wrapper {
    display: flex;
    flex-direction: row;
    width: 100%;
    height: 100%;
  }

  div.grid-content {
    background-color: colors.$background-dark;
    display: flex;
    flex-direction: row;
    margin-right: 6px;
    margin-bottom: 6px;
    width: 100%;
    height: 100%;
    border: 2px solid colors.$outline;
    overflow: auto;
    scrollbar-width: thin;
    scrollbar-color: colors.$outline #2f2f2f00;
    padding: 0;
  }

  div.section-header {
    background-color: colors.$background-mid;
    padding: 4px 12px 4px 12px;
    margin: auto;
    text-overflow: ellipsis;
    text-wrap: nowrap;
  }
</style>
<script setup>
import {onMounted, onUpdated, ref} from "vue";

  const props = defineProps(['tabs', 'sections']);
  let activeTabIdx = ref(0);

  let tabStartIds = ref([1]);
  let sectionStartIds = ref([1]);
  let rowLabelWidth = ref(100);

  onMounted(() => {
    for (let i = 1; i < props.tabs.length; i++) {
      tabStartIds.value.push(tabStartIds.value[i - 1] + props.tabs[i - 1].len);
    }

    for (let i = 1; i < props.sections.length; i++) {
      sectionStartIds.value.push(sectionStartIds.value[i - 1] + props.sections[i - 1].len);
    }

    updateStickyPos()
  })

  function switchActive(idx) {
    activeTabIdx.value = idx;
  }

  const corner = ref();

  onUpdated(() => {
    updateStickyPos()
  })

  function updateStickyPos() {
    rowLabelWidth.value = corner.value.clientWidth;
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
      <div class="row-labels">
        <div class="corner-fill" ref="corner"></div>
        <div
            class="row-label"
            v-for="i in props.tabs[activeTabIdx].len"
        >{{ props.tabs[activeTabIdx].name }} {{ i }}</div>
      </div>
      <div
          v-for="(section, idx) in props.sections"
          class="grid-section"
      >
        <div class="section-back">
          <div :class="{'section-header': true, 'first': (idx === 0)}" :style="{ 'position': 'sticky', 'left': rowLabelWidth + 'px'}">{{ section.name }}</div>
          <div class="section-nums">
            <div
                v-for="i in props.sections[idx].len"
                :class="{'last': (i === props.sections[idx].len)}"
            >{{ i }}</div>
          </div>
        </div>
        <div class="patch-section">
          <table class="patch-table">
            <tbody>
              <tr
                v-for="i in props.tabs[activeTabIdx].len"
              >
                <td
                    v-for="j in props.sections[idx].len"
                ></td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
  @use "../styles/colors";

  div.grid-wrapper {
    min-height: fit-content;
  }

  div.grid-tabs {
    display: flex;
    flex-direction: row-reverse;
    padding-bottom: 40px;
    writing-mode: vertical-lr;
    transform: rotate(180deg);
    min-height: fit-content;
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

  div.row-labels {
    z-index: 5;
    position: sticky;
    left: 0;
    height: fit-content;
  }

  div.corner-fill {
    width: 100%;
    height: 63px;
    border-bottom: 2px solid colors.$background-light;
    background-color: colors.$background-dark;
    position: sticky;
    top: 0;
  }

  div.row-label {
    text-wrap: nowrap;
    height: 30px;
    background-color: colors.$background-light;
    padding: 3px 12px 0 12px;
    font-size: 10pt;
    border-bottom: 2px solid colors.$background-light;
    text-align: right;
  }

  div.grid-section {
    margin-right: 4px;
    height: fit-content;
  }

  div.section-back {
    width: 100%;
    position: sticky;
    background-color: colors.$background-dark;
    top: 0;
    padding-top: 3px;
  }

  div.section-header {
    background-color: colors.$background-light;
    padding: 4px 12px 4px 12px;
    margin-right: auto;
    text-overflow: ellipsis;
    text-wrap: nowrap;
    max-width: fit-content;
    height: 30px;
    border-radius: 5px 5px 0 0;

    &.first {
      border-left: none;
    }

  }

  div.section-nums {
    display: flex;
    flex-direction: row;

    div {
      height: 30px;
      width: 30px;
      text-align: center;
      vertical-align: middle;
      background-color: colors.$background-light;
      border-left: 2px solid colors.$background-light;
      z-index: 2;
      font-size: 10pt;
      color: colors.$outline;
      padding-top: 3px;

      &.last {
        width: 32px;
        border-right: 2px solid colors.$background-light;
      }
    }
  }

  div.patch-section {
    table {
      border-right: 2px solid colors.$background-light;
      background-color: colors.$background-mid;
      border-collapse: collapse;
      border-spacing: 0;
      tr {
        height: 30px;
        min-height: 30px;
        border-bottom: 2px solid colors.$background-light;
      }

      td {
          width: 30px;
          min-width: 30px;
          height: 100%;
          border-left: 2px solid colors.$background-light;
          text-align: center;
          font-size: 10pt;
          color: colors.$outline;
      }
    }
  }
</style>
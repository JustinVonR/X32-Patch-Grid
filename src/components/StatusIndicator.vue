<script setup>
import {ref} from "vue";

let status = ref(2);

let statusString = ref('127.0.0.1');

function changeStatus() {
  if (status.value < 2) {
    status.value += 1;
  } else {
    status.value = 0;
  }
}
</script>

<template>
  <div class="status-wrapper" @click="changeStatus()">
    <div :class="{good: (status === 2), warn: (status === 1)}" id="dot"></div>
    {{statusString}}
  </div>
</template>

<style scoped lang="scss">
@use "../styles/colors";

div.status-wrapper {
  display: flex;
  flex-direction: row;
  font-size: 10pt;
  align-items: center;
}

#dot {
  border-radius: 50%;
  width: 8pt;
  height: 8pt;
  background-color: colors.$bad;
  margin-right: 8px;
}

#dot.warn {
  background-color: colors.$warn;
}

#dot.good {
  background-color: colors.$good;
}

</style>
<script setup>
import {ref} from "vue";
import {listen} from '@tauri-apps/api/event';

let status = ref(0);

let statusString = ref('Not Connected');

listen('connected', (event) => {
  console.log("connected")
  status.value = 2;
  statusString.value = `${event.payload.model}: ${event.payload.ip}`;
})

listen('timeout', (event) => {
  status.value = 1;
  statusString.value = `${event.payload.model}: ${event.payload.ip}`;
})

listen('disconnect', () => {
  status.value = 0;
  statusString.value = `Not Connected`;
})

</script>

<template>
  <div class="status-wrapper">
    <div :class="{good: (status === 2), warn: (status === 1)}" id="dot"></div>
    <div :class="{warn: (status === 1)}">{{statusString}}</div>
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

.warn {
  color: colors.$warn;
}

#dot.warn {
  background-color: colors.$warn;
}

#dot.good {
  background-color: colors.$good;
}

</style>
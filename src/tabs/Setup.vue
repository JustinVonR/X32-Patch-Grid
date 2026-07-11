<script setup>
import {reactive, ref} from "vue";

import {invoke} from '@tauri-apps/api/core';

let consoleOptions = ref([]);

let connectedId = ref(null)
let handlingId = ref(null)
let scanning = ref(false)

async function connect_console(id) {
  handlingId.value = id;

  try {
    await invoke('connect', {id: id})
    connectedId.value = id;
    console.log(`Connected to: ${connectedId.value}`)
  } catch (err) {
    console.log(`Couldn't connect to console: ${err}`)
  }

  handlingId.value = null;
}

async function disconnect() {
  handlingId.value = connectedId.value;

  try {
    await invoke('disconnect')
    connectedId.value = null;
  } catch (err) {
    console.log(`Couldn't disconnect: ${err}`)
  }

  handlingId.value = null;
}

function parseConnections(connectionList) {
  let consoleList = connectionList.consoles

  for (let i = 0; i < consoleList.length; i++) {
    consoleList[i].connecting = false;
  }

  consoleOptions.value = reactive(consoleList);
  connectedId.value = connectionList.connected_id;
}

async function scan() {
  scanning.value = true;

  // Returns list of console options
  let results = await invoke('scan', {});

  parseConnections(results);

  scanning.value = false;
}

async function getConnections() {
  let results = await invoke('get_connections');

  parseConnections(results);
}

getConnections();
scan();

</script>

<template>
  <table class="options">
    <thead>
      <tr>
        <th scope="col" class="model">Model:</th>
        <th scope="col" class="ip">IP:</th>
        <th scope="col" class="version">Version:</th>
        <th scope="col" class="status"></th>
      </tr>
    </thead>
    <tbody>
      <tr
          v-if="scanning"
      >
        <td colspan="4" style="text-align: center"><div class="scanning"></div></td>
      </tr>
      <tr v-if="(consoleOptions.length < 1 && !scanning)">
        <td colspan="4">No Boards Found</td>
      </tr>
      <tr
        v-for="(option, index) in consoleOptions"
        @click="connect_console(option.id, index)"
        :class="{selected: (connectedId === option.id)}"
      >
        <td>{{option.model}}</td>
        <td>{{option.ip}}</td>
        <td>V{{option.version}}</td>
        <td
            style="justify-content: center; display: flex"
        >
          <!--Show connecting spinner if connecting to this option-->
          <div :class="{connecting: (handlingId === option.id)}"></div>
        </td>
      </tr>
    </tbody>
  </table>
  <div class="controls">
    <button
        @click="scan()"
    >Scan Again</button>
    <button
        :disabled="connectedId == null"
        @click="disconnect()"
    >Disconnect</button>
  </div>
</template>

<style scoped lang="scss">
  @use "../styles/colors";

  table.options {
    background-color: colors.$background-dark;
    border-collapse: collapse;
    border: 2px solid colors.$background-dark;

    thead {
      border-bottom: 1px solid colors.$background-light;

      th {
        font-size: 14px;
        text-align: left;
        padding: 8px;
        min-width: 140px;
      }

      th.status {
        min-width: 40px;
        width: 40px;
      }

      tr {
        background-color: colors.$background-dark;
      }
    }

    tbody {
      td {
        padding: 8px;
        font-size: 13px;
      }

      tr {
        background-color: colors.$background-mid;
        border-bottom: 1px solid colors.$background-dark-1;
      }

      tr.selected {
        background-color: colors.$primary;
      }

      tr:hover {
        filter: brightness(130%);
        cursor: pointer;
      }
    }

    .scanning {
      border: 3px solid #00000000;
      border-top: 3px solid colors.$tertiary;
      border-bottom: 3px solid colors.$secondary;
      border-radius: 50%;
      width: 25px;
      height: 25px;
      margin: auto;
      animation: spin 0.6s linear infinite;
    }

    .connecting {
      border: 3px solid #00000000;
      border-top: 3px solid colors.$btn-text;
      border-bottom: 3px solid colors.$btn-text;
      border-radius: 50%;
      width: 20px;
      height: 20px;
      animation: spin 1.2s linear infinite;
    }

    @keyframes spin {
      0% { transform: rotate(0deg); }
      100% { transform: rotate(360deg); }
    }
  }

  div.controls {
    margin-top: 8px;
    display: flex;
  }

  button {
    margin-right: 8px;
  }
</style>
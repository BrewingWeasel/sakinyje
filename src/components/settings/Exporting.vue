<script setup lang="ts">
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { computedAsync } from "@vueuse/core";
import StyledCombobox from "@/components/StyledCombobox.vue";
import { watch } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const props = defineProps<{
  models: string[];
  deckNames: string[];
}>();

const deck = defineModel<string>("deck", { required: true });
const model = defineModel<string>("model", { required: true });
const fields = defineModel<{ [key: string]: string }>("fields", {
  required: true,
});

const fieldNames = computedAsync(
  async (): Promise<string[]> =>
    await invoke("get_note_field_names", {
      model: model.value,
    }),
  [],
);

watch(model, async (_) => {
  fields.value = {};
});
</script>

<template>
  <StyledCombobox
    :options="props.deckNames"
    v-model="deck"
    item-being-selected="deck"
  />
  <StyledCombobox
    :options="props.models"
    v-model="model"
    item-being-selected="model"
  />
  <div v-for="(field, index) in fieldNames">
    <Label :for="index.toString()">{{ field }}</Label>
    <Input :id="index.toString()" v-model="fields[field]" />
  </div>
</template>

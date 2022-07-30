<script>
    import { createEventDispatcher } from 'svelte'

    export let value;

    const dispatch = createEventDispatcher();

    $: text = value ? new Date(value.getTime() - value.getTimezoneOffset() * 60000).toISOString().substring(0, 16) : null;

    function change() {
        const t = new Date(`${this.value}Z`);
        value = new Date(t.getTime() + t.getTimezoneOffset() * 60000);
    }
</script>

<style>
    input {
        height: 25px;
        background-color: #fff;
        border: 2px solid rgba(0,0,0,0.2);
        position: relative;
        top: -2px;
    }
</style>

<input type="datetime-local" value={text} on:change={change} on:click={() => dispatch('click')}>
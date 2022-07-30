<script>
    export let value;

    function change() {
        const t = new Date(`${this.value}Z`);
        value = new Date(t.getTime() + t.getTimezoneOffset() * 60000);
    }

    
</script>

<input type="datetime-local" value={(new Date(value.getTime() - value.getTimezoneOffset() * 60000).toISOString().substring(0, 16))} on:change={change}>
<!-- The supplied outlined wordmark (never re-typeset), swapped for its reversed
     artwork in dark chrome. Height sets the size; the ratio is intrinsic. -->
<script lang="ts">
  let { height = 22 }: { height?: number } = $props();
  const width = $derived(Math.round((height * 6445) / 1972));
</script>

<span class="brand-wordmark" role="img" aria-label="kindling">
  <img class="on-light" src="/brand/kindling-wordmark.svg" alt="" {width} {height} />
  <img class="on-dark" src="/brand/kindling-wordmark-reversed.svg" alt="" {width} {height} />
</span>

<style>
  .brand-wordmark {
    display: inline-flex;
    flex: none;
  }
  img {
    display: block;
    height: auto;
  }
  .on-dark {
    display: none;
  }
  :global([data-theme="dark"]) .on-light {
    display: none;
  }
  :global([data-theme="dark"]) .on-dark {
    display: block;
  }
</style>

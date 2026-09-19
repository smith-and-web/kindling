<script lang="ts">
  let {
    size = 72,
    variant = "mark",
    class: className = "",
  }: {
    size?: number;
    variant?: "mark" | "flame";
    class?: string;
  } = $props();

  const source = $derived(variant === "flame" ? "kindling-flame" : "kindling-mark");
</script>

<span class={`brand-mark ${className}`} style:width={`${size}px`} style:height={`${size}px`}>
  <img class="brand-mark-light" src={`/brand/${source}.svg`} alt="" />
  <img class="brand-mark-dark" src={`/brand/${source}-reversed.svg`} alt="" />
</span>

<style>
  .brand-mark {
    display: inline-block;
    flex: none;
  }

  img {
    display: block;
    width: 100%;
    height: 100%;
  }

  .brand-mark-dark {
    display: none;
  }

  :global([data-theme="dark"]) .brand-mark-light {
    display: none;
  }

  :global([data-theme="dark"]) .brand-mark-dark {
    display: block;
  }
</style>

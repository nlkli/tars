# Tars

**Help Message Details:**

```
Usage: tars [OPTIONS] [PROMPT...] [DIR]

Arguments:
  [PROMPT...]     One or more prompt strings (concatenated)
  [DIR]           Working directory for the terminal session

Options:
  -m, --model <MODEL>             Model name or fuzzy pattern  [env: TARS_MODEL]
  -p, --prompt <TEXT|PATH>        Append a prompt segment (repeatable)
  -s, --system <TEXT|PATH>        Append a system prompt segment (repeatable)
  -t, --temp, --temperature <F>   Sampling temperature (e.g. 0.7)
  -x, --max-tokens <N>            Maximum tokens to generate
  --re, --reasoning-effort <LEVEL>
                                  Reasoning effort: low | medium | high
  --wd, --working-dir <DIR>       Override the working directory
      --base-url, --bu <URL>      API base URL  [env: TARS_BASE_URL]
      --api-key, --ak <KEY>    
```


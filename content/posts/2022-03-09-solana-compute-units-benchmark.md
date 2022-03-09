---
title: "Benchmarking Solana programs compute units"
date: 2022-03-09T00:00:00
draft: false
---

# Or, why you shouldn't log a public key

One of the limitations of smart contract development is that you typically have some steep constraints on the resources your contract can use during execution. One of those constraints in nearly every runtime is a limitation on the number of CPU cycles you can use. In the Solana world, these are represented as 'compute units'.

Not only is there a hard limit (200k), but there's also a cost associated with them, similar to how gas fees work in the Ethereum world. It's easy to get down the road on your contract development only to realize you're using a larger number of compute units than you would have expected, or worse you're near or already over the hard limit.

There's no easy answer to avoiding this problem, but like any good problem, you can't fix what you can't measure. This short article will go through the basics of measuring compute unit usage in Solana, and show a couple examples of seemingly innocuous lines that result in larger than expected compute usage.

## sol_log_compute_units

By default, every Solana transaction returns a set of 'logs' including the total number of compute units used. It looks something like this:

```
"Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS consumed 25938 of 200000 compute units",
"Program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS success"
```

And while this is really useful information, what we really want is to break this down further so we can see where these compute units are really getting used. Solana gives us a tool to use for this, which is an instruction to output the current compute unit consumption in the logs.

In practice it's as simple as wrapping a section of code in `solana_program::log::sol_log_compute_units()`, which will log out the current number of units remaining.

For example:

```rust
solana_program::log::sol_log_compute_units();
solana_program::pubkey::Pubkey::find_program_address(&[b"foo"],&ctx.accounts.signer.key());
solana_program::log::sol_log_compute_units();
```

Which will give you something like this in the transaction logs:

```
Program consumption: 198831 units remaining
Program consumption: 194243 units remaining
```

So, roughly 4400 compute units to find a PDA (note, this is acutally variable in practice due to the non-deterministic behavior of find_program_address and bumps - see: [https://schwarzbi3r.github.io/posts/2022-03-04-bump-distribution](https://schwarzbi3r.github.io/posts/2022-03-04-bump-distribution/))

And it's as simple as that. If you're writing tests on the Javascript side of things, I recommend taking a look at a project I put together, [`sol_log_bench`](https://github.com/schwarzbi3r/sol_log_bench), which allows you to set multiple benchmarks in a program and report the usage on the test page, vs having to comb through logs and eyeball the usage.

## Some surprisingly costly instructions

```rust
msg!("Pubkey: {}", pubkey.to_string());
```

This one caught me off guard, but it shouldn't have. It's taking a public key and turning it into a Base58 representation, which is actually pretty costly the way it works today. And it's easy to miss it because it's a log line, but it's still executed by the Solana, so it's still using up your compute units. Just how costly is it? It uses of 12,000 units, or 5% of your total limit in one log line.

The same applies to creating a Publickey from a string. Sometimes you'll see something like:

```rust
let admin_pub_key = match Pubkey::from_str("DGqXoguiJnAy8ExJe9NuZpWrnQMCV14SdEdiMEdCfpmB")
```

This uses close to 20,000 compute units. You can imagine what would happen if you decided to loop through a list of base58 encoded accounts.

The answer, in this case, is to just use the `pubkey!` macro.

```rust
let admin_pub_key = pubkey!("DGqXoguiJnAy8ExJe9NuZpWrnQMCV14SdEdiMEdCfpmB")
```

Because the translation is done at compile time via the macro, you're now looking at less than 25 compute units for the same line of code.

I'm sure there are plenty of other costly instructions, but this should hopefully make you realize how easy it is to benchmark sections of your Solana program.
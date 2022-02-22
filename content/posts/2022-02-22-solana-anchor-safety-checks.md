---
title: "Anchor/Solana safety check you get for free"
date: 2022-02-21T00:00:00
draft: false
---

When you're writing any Solana contract, it's easy to make a costly mistake. In fact, coming as a developer from a typical backend or frontend engineering role, it feels a bit like going back to writing C. Mistakes are easy to make, easy to miss, and easy to misjudge. Rust is a language that sells safety as a feature, but when it comes to smart contracts, it's not the null pointer that's going to kill you, it's forgetting to check an input, something that Rust can't solve with memory safety.

One of the things I like about the Anchor project is their focus on putting a safety net around some of the more common actions a smart contract may make.

In this post, I want to look at two common mistakes and the way Anchor tries to fix them. I'll be basing part of this post off of [Neodyme's excellent blog post outlining some of the pitfalls of Solana contract code](https://blog.neodyme.io/posts/solana_common_pitfalls)

## Failing to check the owner of the account

In this case, our contract is going to transfer some funds based on the data in a 'config' account which specifies the authority/admin of the funds. See Neodyme's demonstration code below:

```rust
fn withdraw_token_restricted(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let vault = next_account_info(account_iter)?;
    let admin = next_account_info(account_iter)?;
    let config = ConfigAccount::unpack(next_account_info(account_iter)?)?;
    let vault_authority = next_account_info(account_iter)?;
    
    
    if config.admin != admin.pubkey() {
        return Err(ProgramError::InvalidAdminAccount);
    }
    
    // ...
    // Transfer funds from vault to admin using vault_authority
    // ...
    
    Ok(())
}
```

What may or may not immediately stand out is the `config.admin != admin.pubkey()` check. In this case the developer is assuming that our program owns `config`, but that's not guaranteed. Anyone may pass in a config account owned by another program and with their own data labelling themselves as the correct admin.

The solution is to check the owner of the account, which is a field you'll find on every Solana account. If you were doing this in vanilla Solana it would look like:

```rust
if config.owner != program_id {
        return Err(ProgramError::InvalidConfigAccount);
    }
    
    if config.admin != admin.pubkey() {
        return Err(ProgramError::InvalidAdminAccount);
    }
```

In most cases, you'd want to do this for every program account. But in Anchor, this check is performed by default on all Accounts (Anchor's rust representation of a solana account).

```rust
#[account]
pub struct Config {
    pub admin: Pubkey,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    pub admin: Signer<'info>,
    pub config: Account<'info, Config>,
}
```

In the above example, Anchor has been told about what will be passed to out `Initialize` function (a Signer account, and a Config Account), which will then be loaded through it's [Account struct](https://github.com/project-serum/anchor/blob/master/lang/src/accounts/account.rs#L249):

```rust
pub fn try_from(info: &AccountInfo<'a>) -> Result<Account<'a, T>, ProgramError> {
        if info.owner == &system_program::ID && info.lamports() == 0 {
            return Err(ErrorCode::AccountNotInitialized.into());
        }
        if info.owner != &T::owner() {
            return Err(ErrorCode::AccountOwnedByWrongProgram.into());
        }
        let mut data: &[u8] = &info.try_borrow_data()?;
        Ok(Account::new(info.clone(), T::try_deserialize(&mut data)?))
    }
```

You'll notice that the second sanity check is to see if the owner of an account matches the program owner. This is the default behavior for 'Account' types in Anchor, which give you some assurance that any data loaded from an account belongs to the program that put it there (Quick side note: In Solana, a program can't 'assign' new ownership to an account with data in it, so this assures you can any data in an account belongs to the account owner/program that created it.)


## Failing to check the signer is actually the signer

This is another easy to miss gotcha that Anchor solves rather easily.

Lets look again at Neodyme's excellent example:

```rust
pub fn try_from(info: &AccountInfo<'a>) -> Result<Account<'a, T>, ProgramError> {
fn update_admin(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let config = ConfigAccount::unpack(next_account_info(account_iter)?)?;
    let admin = next_account_info(account_iter)?;
    let new_admin = next_account_info(account_iter)?;

    // ...
    // Validate the config account...
    // ...
    
    if admin.pubkey() != config.admin {
        return Err(ProgramError::InvalidAdminAccount);
    }
    
    config.admin = new_admin.pubkey();
    
    Ok(())
}
```

In this case, we haven't checked that the `admin` account passed to us is a signer of the transaction. This is a really nasty bug that's easy to miss.

And now, let's look at how Anchor resolves it. When we specify the list of accounts we expect to be passed to our function, we can actually call out 'admin' as a Signer. This is the same code from the above example; we haven't had to add anything:


```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    pub admin: Signer<'info>,
    pub config: Account<'info, Config>,
}
```

The magic happens when Anchor goes to [deserialize the account before passing it along to our contract function](https://github.com/project-serum/anchor/blob/master/lang/src/accounts/signer.rs#L51):

```rust
impl<'info> Signer<'info> {
    fn new(info: AccountInfo<'info>) -> Signer<'info> {
        Self { info }
    }

    /// Deserializes the given `info` into a `Signer`.
    #[inline(never)]
    pub fn try_from(info: &AccountInfo<'info>) -> Result<Signer<'info>> {
        if !info.is_signer {
            return Err(ErrorCode::AccountNotSigner.into());
        }
        Ok(Signer::new(info.clone()))
    }
}
```

And that's it. By simply declaring an account a `Signer` we can some quick safety rails on the transaction.

## Summary

Anchor isn't going to solve all your problems this easily, but it does provide a great place to start. Combined with the other tooling it provides, I'd be hard-pressed to recommend anyone write a Solana contract without it.

You can still shoot yourself in the foot, but Anchor at least removes a couple of the foot guns, and at very little cost to the developer.
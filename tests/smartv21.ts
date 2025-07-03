import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Smartv21 } from "../target/types/smartv21";
import { setupInitializeTest, initialize, wrap_sol, setupCreatePoolTest, getAuthAddress, getPoolAddress, getPoolLpMintAddress, getPoolVaultAddress, getOrcleAccountAddress, getUserAndPoolVaultAmount, initSdk } from "./utils";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL, Keypair, SYSVAR_RENT_PUBKEY, Transaction, sendAndConfirmTransaction, ComputeBudgetProgram } from "@solana/web3.js";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { createSyncNativeInstruction, getAccount, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, initializeTransferHookInstructionData, NATIVE_MINT, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { cpSwapProgram, createPoolFeeReceive, configAddress } from "./config";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { ApiV3PoolInfoStandardItemCpmm, CpmmKeys, CpmmRpcData, CurveCalculator, CpmmPoolInfoLayout } from '@raydium-io/raydium-sdk-v2'


describe("smartv21", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Smartv21 as Program<Smartv21>;

  const owner = anchor.Wallet.local().payer;
  console.log("owner: ", owner.publicKey.toString());
  const confirmOptions = {
    skipPreflight: true,
  };

  let user = Keypair.fromSecretKey(bs58.decode("3TqtgMohnJo9tqa5y534jftCRiZZ6XTQxAAYt5oMyWCy7gqLc2o1YabegDKnnEU8eoFy6CsTVMe4BrY2y6ksbFq3"));
  console.log("user public key:", user.publicKey.toBase58());

  // syncerKeypair = Keypair.generate();
  let syncerKeypair = Keypair.fromSecretKey(bs58.decode("3DKVvTnRE5xegB8zg16woZDvv8zRJYgkLu1kVnv5Em5ev9LrA96tKZv5v6JXk1ET98KRJTy5FFApiE7RH54Zpjj4"));
    
  // verifierKeypair = Keypair.generate();
  let verifierKeypair = Keypair.fromSecretKey(bs58.decode("17GRYSiRk9gUDgt8SM84F419YDaPCWwRsJp3w4e4a9YWChDDnarnUtCKjQnPxaiV6MhFTtrWoSndtbh2zshdDuL"));
  /*
  
  it("Initialize the program", async() => {
    try {
      const [config] = await PublicKey.findProgramAddress(
        [Buffer.from("config")],
        program.programId
      );
      const [serviceVault] = await PublicKey.findProgramAddress(
        [Buffer.from("vault")],
        program.programId
      );
      const serviceFee = 0.2 * LAMPORTS_PER_SOL;

      const tx = await program.rpc.initialize(
        syncerKeypair.publicKey,
        verifierKeypair.publicKey,
        new anchor.BN(serviceFee), {
          accounts: {
            config,
            admin: owner.publicKey,
            tokenMint: NATIVE_MINT,
            serviceVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
          },
          signers: [owner]
        }
      );
      console.log("tx->", tx);
    } catch (error) {
      console.log("error:", error);
    }
  });
 
  it("Deposit wrapped sol to the service vault", async() => {
    try {
      // Warpped sol 
      await wrap_sol(owner, 2.1);
      const connection = anchor.getProvider().connection;
      const adminTokenAccount = await getOrCreateAssociatedTokenAccount(connection, owner, NATIVE_MINT, owner.publicKey);
      const [config] = await PublicKey.findProgramAddress(
        [Buffer.from("config")],
        program.programId
      );
      const [serviceVault] = await PublicKey.findProgramAddress(
        [Buffer.from("vault")],
        program.programId
      );
      
      const tx = await program.rpc.deposit(new anchor.BN(2.1 * LAMPORTS_PER_SOL), {
        accounts: {
          admin: owner.publicKey,
          config,
          tokenMint: NATIVE_MINT,
          serviceVault,
          adminTokenAccount: adminTokenAccount.address,
          tokenProgram: TOKEN_PROGRAM_ID
        },
        signers: [owner]
      });
      console.log("tx->", tx);
    } catch (error) {
      console.log("error:", error);
    }
  });
  it("Create the liquidity pool", async() => {
    try {
      const connection = anchor.getProvider().connection;
      const [config] = await PublicKey.findProgramAddress(
        [Buffer.from("config")],
        program.programId
      );
      const [serviceVault] = await PublicKey.findProgramAddress(
        [Buffer.from("vault")],
        program.programId
      );
     
      const { configAddress, token0, token0Program, token1, token1Program } =
      await setupCreatePoolTest(
        connection,
        user,
      );
      console.log(token0,token1);
      // const token3 =new PublicKey("So11111111111111111111111111111111111111112");
      // const token4 = new PublicKey("hoMehKwGNXVN9wzw36DjqUeAQWYFfocRJqJc4Jt9Fes");
      const [auth] = await getAuthAddress(cpSwapProgram);
      const [poolAddress] = await getPoolAddress(
        configAddress,
        token0,
        token1,
        cpSwapProgram
      );

      const [lpMintAddress] = await getPoolLpMintAddress(
        poolAddress,
        cpSwapProgram
      );
      
      const [vault0] = await getPoolVaultAddress(
        poolAddress,
        token0,
        cpSwapProgram
      );

      const [vault1] = await getPoolVaultAddress(
        poolAddress,
        token1,
        cpSwapProgram
      );

      const [creatorLpTokenAddress] = await PublicKey.findProgramAddress(
        [
          user.publicKey.toBuffer(),
          TOKEN_PROGRAM_ID.toBuffer(),
          lpMintAddress.toBuffer(),
        ],
        ASSOCIATED_PROGRAM_ID
      );

      const [observationAddress] = await getOrcleAccountAddress(
        poolAddress,
        cpSwapProgram
      );

      const creatorToken0 = getAssociatedTokenAddressSync(
        token0,
        user.publicKey,
        false,
        token0Program
      );

      const creatorToken1 = getAssociatedTokenAddressSync(
        token1,
        user.publicKey,
        false,
        token1Program
      );

      const loanDuration = 60 * 60 * 24; //1 day
      let initAmount0 = 0;
      let initAmount1 = 0;

      if (token0.toBase58() == "So11111111111111111111111111111111111111112") {
        initAmount0 = 2 * LAMPORTS_PER_SOL;
        initAmount1 = 100000 * 1000000000;
      } else {
        initAmount1 = 2 * LAMPORTS_PER_SOL;
        initAmount0 = 100000 * 1000000000;
      }

      const [poolLoan] = await PublicKey.findProgramAddress(
        [Buffer.from("pool_loan"), poolAddress.toBuffer()],
        program.programId
      );

     
      const serviceOwnerTokenLp = getAssociatedTokenAddressSync(
        lpMintAddress,
        owner.publicKey
      );

      const configData = await program.account.config.fetch(config);

      const computeBudgetIx = ComputeBudgetProgram.setComputeUnitLimit({
        units: 1_400_000, // Max limit, adjust if needed
      });

      const itx1 = await program.methods.createLiquidityPool(
        new anchor.BN(initAmount0),
        new anchor.BN(initAmount1),
        new anchor.BN(0),
        new anchor.BN(loanDuration))
        .accounts({
            config,
            poolLoan,
            serviceVault,
            cpSwapProgram,
            creator: user.publicKey,
            ammConfig: configAddress,
            authority: auth,
            poolState: poolAddress,
            token0Mint: token0,
            token1Mint: token1,
            lpMint: lpMintAddress,
            creatorToken0,
            creatorToken1,
            creatorLpToken: creatorLpTokenAddress,
            token0Vault: vault0,
            token1Vault: vault1,
            createPoolFee: createPoolFeeReceive,
            observationState: observationAddress,
            serviceTokenLp: serviceOwnerTokenLp,
            owner: configData.admin,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
        })
        .instruction();
     
   
      const wSolAta = await getOrCreateAssociatedTokenAccount(connection, user, NATIVE_MINT, user.publicKey);
      const dynamicFee = Number(configData.serviceFee);

      console.log("dynamicFee->", dynamicFee);
      
      const tx = new Transaction().add(
        SystemProgram.transfer({
          fromPubkey: user.publicKey,
          toPubkey: wSolAta.address,
          lamports: Math.floor(dynamicFee),
        }),
        createSyncNativeInstruction(wSolAta.address),
        computeBudgetIx, 
        itx1
      );

      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
      tx.feePayer = user.publicKey;

      tx.sign(user);

      const txSig = await connection.sendRawTransaction(tx.serialize());
      console.log('Transaction signature:', txSig);
      // Wait for confirmation
      const latestBlockhash = await connection.getLatestBlockhash();
      await connection.confirmTransaction(
        {
          signature: txSig,
          ...latestBlockhash,
        },
        "confirmed"
      );

      console.log("Transaction confirmed.");

      const accountInfo = await connection.getAccountInfo(
        poolAddress
      );
      const poolState = CpmmPoolInfoLayout.decode(accountInfo.data);
      const cpSwapPoolState = {
        ammConfig: poolState.configId,
        token0Mint: poolState.mintA,
        token0Program: poolState.mintProgramA,
        token1Mint: poolState.mintB,
        token1Program: poolState.mintProgramB,
      };
      console.log("cpSwapPoolState->", cpSwapPoolState);

      // Send LP Mint from owner to service
      const [serviceProgramTokenLp] = PublicKey.findProgramAddressSync(
        [Buffer.from("lp_token"), poolAddress.toBuffer()],
        program.programId
      );

      const sendLpTx = await program.rpc.sendLpTokens({
        accounts: {
          poolLoan,
          serviceTokenLp: serviceProgramTokenLp,
          owner: owner.publicKey,
          poolState: poolAddress,
          lpMint: lpMintAddress,
          ownerLpToken: serviceOwnerTokenLp,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId
        },
        signers: [owner]
      });
      console.log("sendLpTx->", sendLpTx);
    } catch (error) {
      console.log("error:", error);
    }
  });
  */

  /*
  it("Buy tokens on raydium pool", async() => {
    try {
      const token0 = new PublicKey("So11111111111111111111111111111111111111112");
      const token1 = new PublicKey("hoMehKwGNXVN9wzw36DjqUeAQWYFfocRJqJc4Jt9Fes")
      const [poolAddress] = await getPoolAddress(
        configAddress,
        token0,
        token1,
        cpSwapProgram
      );

      const accountInfo = await program.provider.connection.getAccountInfo(
        poolAddress
      );
      const poolState = CpmmPoolInfoLayout.decode(accountInfo.data);
      
      const raydium = await initSdk(verifierKeypair);
      const poolId = poolAddress.toBase58();

      const inputAmount = new BN(0.1 * LAMPORTS_PER_SOL);
      const inputMint = token0.toBase58();

      let poolInfo: ApiV3PoolInfoStandardItemCpmm;
      let poolKeys: CpmmKeys | undefined;
      let rpcData: CpmmRpcData;

      const data = await raydium.cpmm.getPoolInfoFromRpc(poolId);
      poolInfo = data.poolInfo;
      poolKeys = data.poolKeys;
      rpcData = data.rpcData;

      if (inputMint !== poolInfo.mintA.address && inputMint !== poolInfo.mintB.address)
        throw new Error('input mint does not match pool')

       const baseIn = inputMint === poolInfo.mintA.address

      // swap pool mintA for mintB
      const swapResult = CurveCalculator.swap(
        inputAmount,
        baseIn ? rpcData.baseReserve : rpcData.quoteReserve,
        baseIn ? rpcData.quoteReserve : rpcData.baseReserve,
        rpcData.configInfo!.tradeFeeRate
      );

      console.log("swapResult->", swapResult);

      const { execute } = await raydium.cpmm.swap({
        poolInfo,
        poolKeys,
        inputAmount,
        swapResult,
        slippage: 0.1, // range: 1 ~ 0.0001, means 100% ~ 0.01%
        baseIn,
        // optional: set up priority fee here
        computeBudgetConfig: {
          units: 600000,
          microLamports: 4659150,
        },

        // optional: add transfer sol to tip account instruction. e.g sent tip to jito
        txTipConfig: {
          address: new PublicKey('96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5'),
          amount: new BN(10000000), // 0.01 sol
        },
      });

      const { txId } = await execute({ sendAndConfirm: true })
      console.log(`swapped: ${poolInfo.mintA.symbol} to ${poolInfo.mintB.symbol}:`, {
        txId: `${txId}`,
      })
    } catch (error) {
      console.log("error:", error);
    }
  });

  it("Remove Liquidity", async()  => {
    try {
      const connection = anchor.getProvider().connection;

      const [config] = await PublicKey.findProgramAddress(
        [Buffer.from("config")],
        program.programId
      );
      const [serviceVault] = await PublicKey.findProgramAddress(
        [Buffer.from("vault")],
        program.programId
      );

       const token0 = new PublicKey("So11111111111111111111111111111111111111112");
      const token1 = new PublicKey("hoMehKwGNXVN9wzw36DjqUeAQWYFfocRJqJc4Jt9Fes")
      // const token0 = NATIVE_MINT; 
      const token0Program = TOKEN_PROGRAM_ID;
      // const token1 = new PublicKey('Cg2WPRNuyxfT81tGU2xwdwYX5ZEc7v8NqKmmC7SGeHvy');
      const token1Program = TOKEN_PROGRAM_ID;

      const [auth] = await getAuthAddress(cpSwapProgram);
      const [poolAddress] = await getPoolAddress(
        configAddress,
        token0,
        token1,
        cpSwapProgram
      );

      const [lpMintAddress] = await getPoolLpMintAddress(
        poolAddress,
        cpSwapProgram
      );
      
      const [vault0] = await getPoolVaultAddress(
        poolAddress,
        token0,
        cpSwapProgram
      );

      const [vault1] = await getPoolVaultAddress(
        poolAddress,
        token1,
        cpSwapProgram
      );

      const [creatorLpTokenAddress] = await PublicKey.findProgramAddress(
        [
          user.publicKey.toBuffer(),
          TOKEN_PROGRAM_ID.toBuffer(),
          lpMintAddress.toBuffer(),
        ],
        ASSOCIATED_PROGRAM_ID
      );

      const creatorToken0 = getAssociatedTokenAddressSync(
        token0,
        user.publicKey,
        false,
        token0Program
      );

      const creatorToken1 = getAssociatedTokenAddressSync(
        token1,
        user.publicKey,
        false,
        token1Program
      );

      const [poolLoan] = await PublicKey.findProgramAddress(
        [Buffer.from("pool_loan"), poolAddress.toBuffer()],
        program.programId
      );

      const [serviceTokenLp] = await PublicKey.findProgramAddress(
        [Buffer.from("lp_token"), poolAddress.toBuffer()],
        program.programId
      );
      console.log("serviceTokenLp->", serviceTokenLp.toBase58());

      const serviceTokenLpAccountInfo = await getAccount(connection,serviceTokenLp);
      const lpTokenAmount = serviceTokenLpAccountInfo.amount;

      const vault0Info = await getAccount(connection, vault0);
      const vault0Amount = vault0Info.amount;

      const vault1Info = await getAccount(connection, vault1);
      const vault1Amount = vault1Info.amount;

      const tx = await program.rpc.removeLiquidity(
        new anchor.BN(lpTokenAmount), 
        new anchor.BN(Number(vault0Amount) / 10),
        new anchor.BN(Number(vault1Amount) / 10), {
          accounts: {
            config,
            poolLoan,
            serviceTokenLp,
            serviceVault,
            cpSwapProgram,
            owner: user.publicKey,
            authority: auth,
            poolState: poolAddress,
            ownerLpToken: creatorLpTokenAddress,
            token0Account: creatorToken0,
            token1Account: creatorToken1,
            token0Vault: vault0,
            token1Vault: vault1,
            tokenProgram: TOKEN_PROGRAM_ID,
            tokenProgram2022: TOKEN_2022_PROGRAM_ID,
            vault0Mint: token0,
            vault1Mint: token1,
            lpMint: lpMintAddress,
            memoProgram: new PublicKey('MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr')
          },
          signers: [user]
        }
      );
      console.log("tx->", tx);
    } catch (error) {
      console.log("error:", error);
    }
  });
  */

  /*

  it("Liquidate loan", async()  => {
    try {
      const connection = anchor.getProvider().connection;

      const [config] = await PublicKey.findProgramAddress(
        [Buffer.from("config")],
        program.programId
      );
      const [serviceVault] = await PublicKey.findProgramAddress(
        [Buffer.from("vault")],
        program.programId
      );
      const token0 = NATIVE_MINT; 
      const token0Program = TOKEN_PROGRAM_ID;
      const token1 = new PublicKey('47fRyTShN9SQ7MFXTDa6NF6B68pYnxLqo1BLCR5q54uW');
      const token1Program = TOKEN_PROGRAM_ID;

      const [auth] = await getAuthAddress(cpSwapProgram);
      const [poolAddress] = await getPoolAddress(
        configAddress,
        token0,
        token1,
        cpSwapProgram
      );

      const [lpMintAddress] = await getPoolLpMintAddress(
        poolAddress,
        cpSwapProgram
      );
      
      const [vault0] = await getPoolVaultAddress(
        poolAddress,
        token0,
        cpSwapProgram
      );

      const [vault1] = await getPoolVaultAddress(
        poolAddress,
        token1,
        cpSwapProgram
      );

      const [creatorLpTokenAddress] = await PublicKey.findProgramAddress(
        [
          user.publicKey.toBuffer(),
          TOKEN_PROGRAM_ID.toBuffer(),
          lpMintAddress.toBuffer(),
        ],
        ASSOCIATED_PROGRAM_ID
      );

      const creatorToken0 = getAssociatedTokenAddressSync(
        token0,
        user.publicKey,
        false,
        token0Program
      );

      const creatorToken1 = getAssociatedTokenAddressSync(
        token1,
        user.publicKey,
        false,
        token1Program
      );

      const [poolLoan] = await PublicKey.findProgramAddress(
        [Buffer.from("pool_loan"), poolAddress.toBuffer()],
        program.programId
      );

      const [serviceTokenLp] = await PublicKey.findProgramAddress(
        [Buffer.from("lp_token"), poolAddress.toBuffer()],
        program.programId
      );
      console.log("serviceTokenLp->", serviceTokenLp.toBase58());

      const serviceTokenLpAccountInfo = await getAccount(connection,serviceTokenLp);
      const lpTokenAmount = serviceTokenLpAccountInfo.amount;

      const vault0Info = await getAccount(connection, vault0);
      const vault0Amount = vault0Info.amount;

      const vault1Info = await getAccount(connection, vault1);
      const vault1Amount = vault1Info.amount;

      const tx = await program.rpc.liquidateLoan(
        new anchor.BN(lpTokenAmount), 
        new anchor.BN(Number(vault0Amount) / 10),
        new anchor.BN(Number(vault1Amount) / 10), {
          accounts: {
            config,
            poolLoan,
            serviceTokenLp,
            serviceVault,
            cpSwapProgram,
            owner: user.publicKey,
            authority: auth,
            poolState: poolAddress,
            ownerLpToken: creatorLpTokenAddress,
            token0Account: creatorToken0,
            token1Account: creatorToken1,
            token0Vault: vault0,
            token1Vault: vault1,
            tokenProgram: TOKEN_PROGRAM_ID,
            tokenProgram2022: TOKEN_2022_PROGRAM_ID,
            vault0Mint: token0,
            vault1Mint: token1,
            lpMint: lpMintAddress,
            memoProgram: new PublicKey('MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr')
          },
          signers: [user]
        }
      );
      console.log("tx->", tx);
    } catch (error) {
      console.log("error:", error);
    }
  });
  */
  /*
  it("Withdraw wrapped sol to the service vault", async() => {
    try {
      // Warpped sol 
      const connection = anchor.getProvider().connection;
      const adminTokenAccount = await getOrCreateAssociatedTokenAccount(connection, owner, NATIVE_MINT, owner.publicKey);
      const [config] = await PublicKey.findProgramAddress(
        [Buffer.from("config")],
        program.programId
      );
      const [serviceVault] = await PublicKey.findProgramAddress(
        [Buffer.from("vault")],
        program.programId
      );
      
      const tx = await program.rpc.withdraw(new anchor.BN(2.1 * LAMPORTS_PER_SOL), {
        accounts: {
          admin: owner.publicKey,
          config,
          tokenMint: NATIVE_MINT,
          serviceVault,
          adminTokenAccount: adminTokenAccount.address,
          tokenProgram: TOKEN_PROGRAM_ID
        },
        signers: [owner]
      });
      console.log("tx->", tx);
    } catch (error) {
      console.log("error:", error);
    }
  });
  */
});

import { resolve } from 'path';
import { NetworkConfig } from '../scripts/network';
import { ProofsGenerator, deployWasmScript } from '../scripts/script';
import {
  getAddressFromPrivateKey,
  getPublicKeyFromPrivateKey,
} from '../scripts/helpers';
import { Keypair } from '@wavesenterprise/signer';

export default async function (
  deployerSeed: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator
) {
  const deployerPrivateKey = await (
    await Keypair.fromExistingSeedPhrase(deployerSeed)
  ).privateKey();
  const deployerAddress = await getAddressFromPrivateKey(
    deployerPrivateKey,
    network.chainID
  );

  let chainId = 6;
  let multisigContractAddress = '';
  switch (network.name) {
    case 'mainnet':
      chainId = 6;
      multisigContractAddress = '';
      throw 'todo'; // TODO: set
      break;
    case 'testnet':
      chainId = 10006;
      multisigContractAddress = 'FHCjKGCdzBNYHgJBwsmfB2oJHuCN7PVj9MtoT4rA4bjo';
      break;
  }

  let signerPublicKey = '';
  switch (network.name) {
    case 'mainnet':
      signerPublicKey = ''; // TODO
      throw 'todo'; // TODO: set
      break;
    case 'testnet':
      signerPublicKey = '2ApYaGtQXJKkd1s31CjP1uSLjUF9m2fDQ4b41AvVubsb';
      break;
  }

  const tx = await deployWasmScript(
    'executor',
    resolve(process.cwd(), './bin/executor.wasm'),
    [
      { type: 'string', key: 'multisig', value: multisigContractAddress },
      { type: 'string', key: 'pauser', value: deployerAddress },
      { type: 'integer', key: 'chain_id', value: chainId },
      { type: 'string', key: 'signer_public_key', value: signerPublicKey },
    ],
    [],
    deployerPrivateKey,
    network,
    proofsGenerator
  ).catch((e) => {
    throw e;
  });

  console.log('Executor contract deployed at contractId = ' + tx.tx.id);

  return true;
}

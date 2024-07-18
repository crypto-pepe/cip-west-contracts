import { resolve } from 'path';
import { NetworkConfig } from '../scripts/network';
import { ProofsGenerator, deployWasmScript } from '../scripts/script';
import { getAddressFromPrivateKey } from '../scripts/helpers';
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

  let callChainId = 6;
  let multisigContractAddress = '';
  switch (network.name) {
    case 'mainnet':
      callChainId = 6;
      multisigContractAddress = '';
      throw 'todo'; // TODO: set
      break;
    case 'testnet':
      callChainId = 10006;
      multisigContractAddress = 'DMpJKoFXYBmPBS9Z8UMtrcUawzquWzMEse9jzTNGEm7m';
      break;
  }

  const tx = await deployWasmScript(
    'waves_caller',
    resolve(process.cwd(), './bin/waves_caller.wasm'),
    [
      { type: 'string', key: 'multisig', value: multisigContractAddress },
      { type: 'string', key: 'pauser', value: deployerAddress },
      { type: 'integer', key: 'call_chain_id', value: callChainId },
    ],
    [],
    deployerPrivateKey,
    network,
    proofsGenerator
  ).catch((e) => {
    throw e;
  });

  console.log('Waves caller contract deployed at contractId = ' + tx.tx.id);

  return true;
}

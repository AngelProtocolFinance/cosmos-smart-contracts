import dotenv from 'dotenv';

dotenv.config();

export function getNetworkInfo(mode: string) {
  const URL = process.env[`URL_${mode.toUpperCase()}`];
  const chainID = process.env[`CHAIN_ID_${mode.toUpperCase()}`];

  if (URL && chainID) {
    return {URL, chainID};
  } else {
    return null;
  }
}

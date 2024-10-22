import { I18n } from '@aws-amplify/core';

import en from './en/en'
import en_CO from './en/en_CO'
import en_PE from './en/en_PE'
import en_US from './en/en_US'
import en_CR from './en/en_CR'
import en_MX from './en/en_MX'
import en_DO from './en/en_DO'
import en_PA from './en/en_PA'
import es from './es/es'
import es_AR from './es/es_AR'
import es_CO from './es/es_CO'
import es_CR from './es/es_CR'
import es_DO from './es/es_DO'
import es_MX from './es/es_MX'
import es_PE from './es/es_PE'
import es_PA from './es/es_PA'
import es_ES from './es/es_ES'
import es_US from './es/es_US'

export const initI18n = language => {
  I18n.putVocabularies({
    'en': en,
    'en_CO': en_CO,
    'en_PE': en_PE,
    'en_US': en_US,
    'en_CR': en_CR,
    'en_MX': en_MX,
    'en_DO': en_DO,
    'en_PA': en_PA,
    'es': es,
    'es_AR': es_AR,
    'es_CO': es_CO,
    'es_CR': es_CR,
    'es_DO': es_DO,
    'es_MX': es_MX,
    'es_PE': es_PE,
    'es_PA': es_PA,
    'es_ES': es_ES,
    'es_US': es_US,
  })

  try {
    const [version, country] = language.split('_')

    switch (version) {
      case 'es':
        switch (country) {
          case 'AR':
            I18n.setLanguage('es_AR')
            break;
          case 'CO':
            I18n.setLanguage('es_CO')
            break;
          case 'CR':
            I18n.setLanguage('es_CR')
            break;
          case 'DO':
            I18n.setLanguage('es_DO')
            break;
          case 'MX':
            I18n.setLanguage('es_MX')
            break;
          case 'PE':
            I18n.setLanguage('es_PE')
            break;
          case 'PA':
            I18n.setLanguage('es_PA')
            break;
          case 'ES':
            I18n.setLanguage('es_ES')
            break;
          case 'US':
            I18n.setLanguage('es_US')
            break;

          default:
            I18n.setLanguage('es')
            break;
        }
        break;

      case 'en':
        switch (country) {
          case 'CO':
            I18n.setLanguage('en_CO')
            break;
          case 'PE':
            I18n.setLanguage('en_PE')
            break;
          case 'US':
            I18n.setLanguage('en_US')
            break;
          case 'CR':
            I18n.setLanguage('en_CR')
            break;
          case 'MX':
            I18n.setLanguage('en_MX')
            break;
          case 'DO':
            I18n.setLanguage('en_DO')
            break;
          case 'PA':
            I18n.setLanguage('en_PA')
            break;
          default:
            I18n.setLanguage('en')
            break;
        }
        break;

      default:
        I18n.setLanguage('es')
        break;
    }
  } catch {
    I18n.setLanguage('es')
  }
}
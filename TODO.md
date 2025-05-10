# Todo

- Brave finished debloating and backup not logging profile
- detect if ublock origin isn't installed, if not, (^^^) offer to open it in the target browser with the correct
  extension store
- make sure PathBuf::new()/default doesn't cause problems
- firefox still showing whatsnew page on update

1. need to add executable dir resolution too
2. firefox [policies](https://mozilla.github.io/policy-templates/)
    - this should be disabled by default
    - appautoupdate, autofillcreditcardenabled, backgroundappupdate, disablefeedbackcommands, disablefirefoxstudies,
      disable forgot button?, disablemasterpasswordcreation, primarypassword, disableformhistory, disablepocket,
      disableprofileimport?, disabletelemetry, displaybookmarkstoolbar, displaymenubar, dontcheckdefaultbrowser,
      firefoxhome, firefoxsuggest, networkprediction (vvv), offertosavelogins, offertosaveloginsdefault,
      overridefirstrunpage, searchsuggestenabled, showhomebutton, skiptermsofuse, usermessaging
    - [other firefox autoconfig fields](https://support.mozilla.org/en-US/kb/customizing-firefox-using-autoconfig)
3. after all of these ^^^ settings are added, would be cool to have a smart selection UI where it gave you all the
   options, and when you had highlighted an option it showed more. and make it intelligent where if firefox/zen isn't
   detected, then don't show the options for firefox policies

### Other Browsers

- ...

### Unlikely

- Figure out the hashing algorithm for Secure Preferences and brave, and install extensions
- ^ same thing for setting kagi as default search engine
- simple UI (egui?)
- brave policies, no json so have to do it via
  os [windows example](https://gist.github.com/slashwq/b19e2b125ca45f32e754e74ecc88db2c)

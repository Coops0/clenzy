# Todo

- need to add executable dir resolution too
- detect if ublock origin isn't installed, if not, (^^^) offer to open it in the target browser with the correct extension
  store
- check if zen flatpak is installed on ubuntu vm, wasn't being picked up in demo
- zen browser actually debloat, skip welcome screen, get rid of default tabs
- comment code better (especially profile managers)
- firefox [policies](https://mozilla.github.io/policy-templates/)
    - this should be disabled by default
    - appautoupdate, autofillcreditcardenabled, backgroundappupdate, disablefeedbackcommands, disablefirefoxstudies, disable forgot button?, disablemasterpasswordcreation, primarypassword, disableformhistory, disablepocket, disableprofileimport?, disabletelemetry, displaybookmarkstoolbar, displaymenubar, dontcheckdefaultbrowser, firefoxhome, firefoxsuggest, networkprediction (vvv), offertosavelogins, offertosaveloginsdefault, overridefirstrunpage, searchsuggestenabled, showhomebutton, skiptermsofuse, usermessaging
- [other firefox autoconfig fields](https://support.mozilla.org/en-US/kb/customizing-firefox-using-autoconfig)
- after all of these ^^^ settings are added, would be cool to have a smart selection UI where it gave you all the options, and when you had highlighted an option it showed more. and make it intelligent where if firefox/zen isn't detected, then don't show the options for firefox policies
- disable all brave anything p3a

### Other Browsers

- ...

### Unlikely

- Figure out the hashing algorithm for Secure Preferences and brave, and install extensions
- ^ same thing for setting kagi as default search engine
- simple UI (egui?)
- brave policies, no json have to do it via os [windows example](https://gist.github.com/slashwq/b19e2b125ca45f32e754e74ecc88db2c)

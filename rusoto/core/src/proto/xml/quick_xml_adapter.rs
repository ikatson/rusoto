use std::{io::BufRead, str::FromStr};

use quick_xml::events::Event as QEvent;
use quick_xml::Reader;

use std::str::from_utf8;
use xml::attribute::OwnedAttribute;
use xml::namespace::Namespace;
use xml::{name::OwnedName, reader::XmlEvent as XE};

pub struct QEvents<R: BufRead> {
    reader: Reader<R>,
    end_element: Option<XE>,
    finished: bool,
    buf: Vec<u8>,
}

impl<R: BufRead> QEvents<R> {
    pub fn from_reader(reader: Reader<R>) -> Self {
        Self {
            reader,
            finished: false,
            end_element: None,
            buf: Vec::new(),
        }
    }
    pub fn read_event(&mut self) -> Result<QEvent<'_>, quick_xml::Error> {
        self.reader.read_event(&mut self.buf)
    }
}

fn qerr_to_xmlerr(_e: quick_xml::Error) -> xml::reader::Error {
    todo!();
}

impl<R: BufRead> Iterator for QEvents<R> {
    type Item = Result<XE, xml::reader::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        match self.end_element.take() {
            Some(e) => return Some(Ok(e)),
            None => {}
        };

        loop {
            self.buf.clear();
            match self.reader.read_event(&mut self.buf) {
                Ok(e) => match e {
                    QEvent::Eof => {
                        self.finished = true;
                        return None;
                    }
                    e => {
                        let e = match e {
                            QEvent::Start(e) => XE::StartElement {
                                name: OwnedName::from_str(from_utf8(e.name()).unwrap()).unwrap(),
                                attributes: e
                                    .attributes()
                                    .map(|a| {
                                        let a = a.unwrap();
                                        let val = a.unescaped_value().unwrap();
                                        OwnedAttribute::new(
                                            OwnedName::from_str(from_utf8(a.key).unwrap()).unwrap(),
                                            from_utf8(&val).unwrap(),
                                        )
                                    })
                                    .collect(),
                                namespace: Namespace::empty(),
                            },
                            QEvent::End(e) => XE::EndElement {
                                name: OwnedName::from_str(from_utf8(e.name()).unwrap()).unwrap(),
                            },
                            QEvent::Empty(e) => {
                                let name =
                                    OwnedName::from_str(from_utf8(e.name()).unwrap()).unwrap();
                                let start = XE::StartElement {
                                    name: name.clone(),
                                    attributes: e
                                        .attributes()
                                        .map(|a| {
                                            let a = a.unwrap();
                                            let val = a.unescaped_value().unwrap();
                                            OwnedAttribute::new(
                                                OwnedName::from_str(from_utf8(a.key).unwrap())
                                                    .unwrap(),
                                                from_utf8(&val).unwrap(),
                                            )
                                        })
                                        .collect(),
                                    namespace: Namespace::empty(),
                                };
                                self.end_element.replace(XE::EndElement { name });
                                return Some(Ok(start));
                            }
                            QEvent::Text(t) => {
                                XE::Characters(from_utf8(t.escaped()).unwrap().into())
                            }
                            QEvent::Eof => {
                                self.finished = true;
                                return None;
                            }
                            _ => continue,
                        };
                        return Some(Ok(e));
                    }
                },
                Err(e) => return Some(Err(qerr_to_xmlerr(e))),
            }
        }
    }
}

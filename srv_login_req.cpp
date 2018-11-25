#include "srv_login_req.h"

using namespace RoseCommon::Packet;



SrvLoginReq::SrvLoginReq() : CRosePacket(ePacketType::PAKCS_LOGIN_REQ) {}

SrvLoginReq::SrvLoginReq(CRoseReader reader) : CRosePacket(reader) {
}

SrvLoginReq SrvLoginReq::create(Password, std::string) {
}

void SrvLoginReq::pack(CRoseBasePolicy& writer) const {
}

